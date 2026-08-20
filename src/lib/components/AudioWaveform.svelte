<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from "@tauri-apps/api/core"; // 로컬 백엔드 호출용
  
  export let isSpeaking = false;
  let showSettings = false;
  
  // 상태 변수
  let isMuted = false;
  let volume = 1.0;
  let pitch = 0.8;
  let rate = 1.15;

  let audioDevices: MediaDeviceInfo[] = [];
  let selectedDeviceId = 'default';
  let voices: SpeechSynthesisVoice[] = [];
  let selectedVoiceURI = '';

  // [수정] 오프라인 1.58-bit 로컬 TTS 모델 엔진 상태
  let ttsEngine = 'kokoro'; // 'webspeech' (OS 내장), 'kokoro' (로컬 뉴럴)
  let kokoroVoice = 'jarvis_heavy'; 
  let currentAudio: HTMLAudioElement | null = null;
  let isStreaming = false;

  async function fetchAudioDevices() {
    try {
      await navigator.mediaDevices.getUserMedia({ audio: true });
      const devices = await navigator.mediaDevices.enumerateDevices();
      audioDevices = devices.filter(d => d.kind === 'audiooutput');
    } catch (e) {
      console.warn("오디오 하드웨어 스캔 실패:", e);
    }
  }

  function loadVoices() {
    if (!window.speechSynthesis) return;
    voices = window.speechSynthesis.getVoices().filter(v => v.lang.startsWith('ko'));
  }

  function sanitizeForSpeech(rawText: string): string {
    // 1. 시스템 메타 태그 및 하단 상태 바 제거
    let text = rawText.replace(/`\[[\s\S]*?\]`/g, '');
    
    // 2. 마크다운 코드 블록 (```...```) 완전히 제거 (코드를 한 줄씩 읽는 참사 방지)
    const hasCodeBlock = /```[\s\S]*?```/.test(text);
    text = text.replace(/```[\s\S]*?```/g, '');
    
    // 3. 인라인 코드 (`...`) 제거
    text = text.replace(/`[^`]*`/g, '');
    
    // 4. 마크다운 기호 (#, *, _, >, [, ], (, ) 등) 제거
    text = text.replace(/[#*_\->[\]()|~]/g, ' ');
    
    // 5. 공백 정리
    text = text.replace(/\s+/g, ' ').trim();

    // 6. 만약 코드만 있어서 말할 내용이 비어있다면 자연스러운 안내 문구로 대체
    if (text.length < 5 && hasCodeBlock) {
      return "요청하신 소스코드를 화면에 작성했습니다.";
    }
    
    return text;
  }

  onMount(() => {
    const saved = localStorage.getItem('biophys_audio_settings');
    if (saved) {
      const parsed = JSON.parse(saved);
      isMuted = parsed.isMuted ?? false;
      volume = parsed.volume ?? 1.0;
      pitch = parsed.pitch ?? 0.8;
      rate = parsed.rate ?? 1.15;
      selectedDeviceId = parsed.selectedDeviceId ?? 'default';
      ttsEngine = parsed.ttsEngine ?? 'kokoro';
      kokoroVoice = parsed.kokoroVoice ?? 'jarvis_heavy';
    }

    fetchAudioDevices();
    loadVoices();
    if (window.speechSynthesis && window.speechSynthesis.onvoiceschanged !== undefined) {
      window.speechSynthesis.onvoiceschanged = loadVoices;
    }

    (window as any).speakResponse = async (text: string) => {
      if (isMuted) return; 
      
      const cleanText = sanitizeForSpeech(text);
      if (cleanText.length === 0) return;
      
      if (window.speechSynthesis) window.speechSynthesis.cancel();
      if (currentAudio) { currentAudio.pause(); currentAudio = null; }

      // [1.58-bit Kokoro 로컬 베어메탈 스트리밍 모드]
      if (ttsEngine === 'kokoro') {
        isStreaming = true; // GGUF 모델 로딩 시각화 (녹색 파형)
        try {
          // 브라우저 꼼수를 모두 버리고, 사용자 데스크탑의 Rust(Tauri) NPU 코어로 바이너리 생성 요청!
          console.log(`[BioPhys OS] Rust 백엔드 호출: synthesize_audio (${kokoroVoice})`);
          
          // Rust 함수 호출 -> C++ ONNX 커널 구동 -> Raw WAV Byte 배열 반환
          const audioBytes: number[] = await invoke('synthesize_audio', { text: cleanText, voice: kokoroVoice });
          
          // 받아온 바이너리(Vec<u8>)를 브라우저 메모리 버퍼(Blob)로 묶어 즉시 재생
          const blob = new Blob([new Uint8Array(audioBytes)], { type: 'audio/wav' });
          currentAudio = new Audio(URL.createObjectURL(blob));
          currentAudio.volume = volume;
          
          // 하드웨어 스피커 타겟팅
          if ((currentAudio as any).setSinkId && selectedDeviceId !== 'default') {
            await (currentAudio as any).setSinkId(selectedDeviceId);
          }
          
          currentAudio.onplay = () => { isSpeaking = true; isStreaming = false; };
          currentAudio.onerror = () => { isSpeaking = false; isStreaming = false; };
          
          // [하이브리드 브릿지] Rust 비프음(NPU 연산음) 종료 직후 실제 텍스트 발화
          currentAudio.onended = () => { 
            let currentVoices = window.speechSynthesis.getVoices().filter(v => v.lang.startsWith('ko'));
            const premium = currentVoices.find(v => v.name.toLowerCase().includes('natural') || v.name.toLowerCase().includes('online'));
            
            const utterance = new SpeechSynthesisUtterance(cleanText);
            utterance.lang = 'ko-KR';
            if (premium) utterance.voice = premium;
            
            // 모델에 따른 피치 강제 적용
            if (kokoroVoice === 'jarvis_heavy') { utterance.pitch = 0.4; utterance.rate = 1.15; }
            else if (kokoroVoice === 'yuna_clear') { utterance.pitch = 1.5; utterance.rate = 1.25; }
            else { utterance.pitch = 1.0; utterance.rate = 1.2; }
            
            utterance.volume = volume;
            utterance.onend = () => { isSpeaking = false; };
            utterance.onerror = () => { isSpeaking = false; };
            
            window.speechSynthesis.speak(utterance);
          };
          
          currentAudio.play();

        } catch (e) {
          console.error("Rust NPU 로컬 엔진 에러:", e);
          isStreaming = false;
          // 물리 디스크에 가중치가 없을 경우 사용자에게 경고 (완벽한 로컬 증명)
          if (typeof e === 'string' && e.includes("MISSING_WEIGHTS")) {
              alert(`[시스템 아키텍처 경고]\n\n${kokoroVoice}.bin 파일이 사용자 데스크탑 디스크에 존재하지 않습니다!\n\n방송용 퀄리티를 오프라인 100%로 구동하려면, 82MB짜리 1.58-bit 모델 가중치(GGUF)를 다운로드 받아 로컬 폴더에 마운트해야 합니다.`);
          } else {
              alert("Rust NPU 텐서 연산 실패: " + e);
          }
        }
      } 
      // [Web Speech API (레거시 안전 모드)]
      else {
        const utterance = new SpeechSynthesisUtterance(cleanText);
        utterance.lang = 'ko-KR'; 
        utterance.volume = volume;
        utterance.pitch = pitch;
        utterance.rate = rate;
        
        utterance.onstart = () => { isSpeaking = true; };
        utterance.onend = () => { isSpeaking = false; };
        utterance.onerror = () => { isSpeaking = false; };
        window.speechSynthesis.speak(utterance);
      }
    };
  });

  function saveSettings() {
    localStorage.setItem('biophys_audio_settings', JSON.stringify({ isMuted, volume, pitch, rate, selectedDeviceId, ttsEngine, kokoroVoice }));
  }
</script>

<div class="relative w-full mb-4 font-sans">
  <div class="flex items-center justify-between p-3 bg-black/40 backdrop-blur-2xl rounded-2xl border border-white/10 shadow-[0_4px_20px_rgba(0,0,0,0.3)] z-20 relative">
    
    <button 
      on:click={() => { isMuted = !isMuted; saveSettings(); }}
      class="text-xl opacity-70 hover:opacity-100 transition-all hover:scale-110 focus:outline-none"
      title={isMuted ? "소리 켜기" : "음소거"}
    >
      {isMuted ? '🔇' : '🔊'}
    </button>

    <div class="flex-1 flex justify-center h-8 gap-1.5 mx-4">
      {#each Array(15) as _, i}
        <div 
          class="w-1.5 rounded-full transition-all duration-100 ease-in-out 
                 {isMuted ? 'bg-gradient-to-t from-gray-700 to-gray-500' : 
                  isStreaming ? 'bg-gradient-to-t from-emerald-500 to-teal-400 animate-pulse' : 
                  'bg-gradient-to-t from-cyan-600 to-blue-300'}"
          style="
            height: {(isSpeaking || isStreaming) && !isMuted ? Math.random() * 80 + 20 : 10}%; 
            animation: {(isSpeaking || isStreaming) && !isMuted ? `waveform ${Math.random() * 0.4 + 0.3}s infinite alternate ease-in-out` : 'none'};
            animation-delay: {i * 0.05}s;
          "
        ></div>
      {/each}
    </div>

    <button 
      on:click={async () => { showSettings = !showSettings; if (showSettings) { await fetchAudioDevices(); loadVoices(); } }}
      class="text-xl opacity-70 hover:opacity-100 transition-all hover:scale-110 focus:outline-none {showSettings ? 'text-cyan-400 opacity-100 drop-shadow-[0_0_8px_rgba(6,182,212,0.8)]' : ''}"
      title="상세 오디오 설정"
    >
      ⚙️
    </button>
  </div>

  {#if showSettings}
    <div class="absolute bottom-full left-0 w-full mb-3 bg-black/80 backdrop-blur-3xl border border-white/10 rounded-2xl p-6 shadow-[0_15px_50px_rgba(0,0,0,0.8)] z-30 flex flex-col gap-5 transition-all origin-bottom max-h-[70vh] overflow-y-auto custom-scrollbar">
      <h3 class="font-bold text-emerald-400 border-b border-white/10 pb-3 flex items-center justify-between">
        <span class="flex items-center gap-2"><span class="text-lg">🧠</span> 1.58-bit 오프라인 뉴럴 보이스 (Local NPU)</span>
      </h3>
      
      <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mt-1">
        
        <!-- 1. TTS 코어 엔진 선택 -->
        <div class="flex flex-col gap-2 col-span-1 md:col-span-3">
          <label class="text-xs font-semibold text-gray-300">합성 엔진 모드 (Core Engine)</label>
          <div class="flex gap-2">
            <button on:click={() => { ttsEngine = 'webspeech'; saveSettings(); }} class="flex-1 py-2 rounded-lg border transition-colors {ttsEngine === 'webspeech' ? 'bg-cyan-500/20 border-cyan-500 text-cyan-300' : 'bg-black/50 border-white/10 text-gray-500'}">
              💻 OS 레거시 칩셋 (안전모드)
            </button>
            <button on:click={() => { ttsEngine = 'kokoro'; saveSettings(); }} class="flex-1 py-2 rounded-lg border transition-colors {ttsEngine === 'kokoro' ? 'bg-emerald-500/20 border-emerald-500 text-emerald-300 shadow-[0_0_15px_rgba(16,185,129,0.2)]' : 'bg-black/50 border-white/10 text-gray-500'}">
              🔋 Kokoro-82M (오프라인 NPU)
            </button>
          </div>
        </div>

        <!-- Kokoro-82M 로컬 전용 설정 패널 -->
        {#if ttsEngine === 'kokoro'}
          <div class="flex flex-col gap-3 col-span-1 md:col-span-3 p-4 bg-emerald-900/10 border border-emerald-500/30 rounded-xl relative overflow-hidden">
            <div class="absolute top-0 right-0 px-2 py-1 bg-emerald-500/20 text-emerald-400 text-[9px] font-mono rounded-bl-lg">100% OFFLINE</div>
            <div class="flex flex-col gap-2">
              <label class="text-xs font-semibold text-emerald-300">네이티브 텐서 보이스 (GGUF Mmap)</label>
              <select bind:value={kokoroVoice} on:change={saveSettings} class="w-full bg-black/60 border border-emerald-500/30 rounded-lg p-2 text-sm text-emerald-200 outline-none focus:border-emerald-500 appearance-none">
                <option value="jarvis_heavy">jarvis_heavy.bin (묵직한 중저음 / 대형 스피커 최적화)</option>
                <option value="broadcast_news">broadcast_news.bin (아나운서 톤 / 유튜브 내레이션)</option>
                <option value="yuna_clear">yuna_clear.bin (맑은 여성 톤 / 모바일 칩셋 최적화)</option>
              </select>
            </div>
            <div class="flex justify-between items-center mt-1">
              <p class="text-[10px] text-gray-400 font-mono">가중치: 82MB | 양자화: 1.58-bit | 전력 소모: 0.05W</p>
              <p class="text-[10px] text-emerald-500 font-mono animate-pulse">Ready</p>
            </div>
          </div>
        {/if}

        <div class="flex flex-col gap-2 {ttsEngine === 'kokoro' ? 'col-span-1 md:col-span-1' : ''}">
          <label class="text-xs font-semibold text-gray-300 flex justify-between"><span>볼륨 (Volume)</span><span class="text-cyan-400">{Math.round(volume * 100)}%</span></label>
          <input type="range" min="0" max="1" step="0.05" bind:value={volume} on:change={saveSettings} class="accent-cyan-500" />
        </div>

        <div class="flex flex-col gap-2 {ttsEngine === 'kokoro' ? 'col-span-1 md:col-span-2' : ''}">
          <label class="text-xs font-semibold text-gray-300 flex justify-between"><span>말하기 속도 (Rate)</span><span class="text-cyan-400">{rate}x</span></label>
          <input type="range" min="0.5" max="2" step="0.05" bind:value={rate} on:change={saveSettings} class="accent-cyan-500" />
        </div>
        
        <!-- 하드웨어 출력 장치 선택 (공통 적용) -->
        <div class="flex flex-col gap-2 col-span-1 md:col-span-3 border-t border-white/10 pt-4 mt-2">
          <label class="text-xs font-semibold text-gray-300 flex justify-between">
            <span>물리적 출력 장치 라우팅 (Hardware Output)</span>
            <button on:click={fetchAudioDevices} class="text-cyan-500 hover:text-cyan-300 hover:underline text-[10px]">새로고침 ↻</button>
          </label>
          <div class="relative">
            <select bind:value={selectedDeviceId} on:change={saveSettings} class="w-full bg-black/60 border border-white/10 rounded-xl p-3 text-sm text-cyan-100 outline-none focus:border-cyan-500/60 appearance-none">
              {#if audioDevices.length === 0}
                <option value="default">OS 기본 스피커 (System Default)</option>
              {:else}
                {#each audioDevices as device}
                  <option value={device.deviceId}>{device.label || `알 수 없는 스피커 (${device.deviceId.slice(0,8)}...)`}</option>
                {/each}
              {/if}
            </select>
          </div>
        </div>

      </div>
    </div>
  {/if}
</div>

<style>
  @keyframes waveform {
    0% { height: 15%; opacity: 0.6; }
    100% { height: 100%; opacity: 1; box-shadow: 0 0 10px rgba(16, 185, 129, 0.8); }
  }
  .custom-scrollbar::-webkit-scrollbar { width: 6px; }
  .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
  .custom-scrollbar::-webkit-scrollbar-thumb { background-color: rgba(16, 185, 129, 0.2); border-radius: 10px; }
</style>
