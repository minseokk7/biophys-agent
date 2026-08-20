<script lang="ts">
  import { onMount } from 'svelte';
  
  export let isSpeaking = false;
  let showSettings = false;
  
  // 사용자 커스텀 설정 상태
  let isMuted = false;
  let volume = 1.0;
  let pitch = 0.8;
  let rate = 1.15;

  onMount(() => {
    // 앱이 켜질 때 로컬 스토리지에서 이전 설정 불러오기
    const saved = localStorage.getItem('biophys_audio_settings');
    if (saved) {
      const parsed = JSON.parse(saved);
      isMuted = parsed.isMuted ?? false;
      volume = parsed.volume ?? 1.0;
      pitch = parsed.pitch ?? 0.8;
      rate = parsed.rate ?? 1.15;
    }

    // TTS 오디오 출력 바인딩
    (window as any).speakResponse = (text: string) => {
      if (isMuted) return; // 음소거 상태면 스피커를 켜지 않음
      
      const cleanText = text.replace(/`\[ ⏱️.*?\]`/g, '').replace(/[\*\[\]\n_]/g, ' ').trim();
      if (!window.speechSynthesis || cleanText.length === 0) return;
      
      window.speechSynthesis.cancel(); 
      const utterance = new SpeechSynthesisUtterance(cleanText);
      utterance.lang = 'ko-KR'; 
      
      // 사용자가 조절한 슬라이더 값을 즉각 반영
      utterance.volume = volume;
      utterance.pitch = pitch;
      utterance.rate = rate;
      
      utterance.onstart = () => { isSpeaking = true; };
      utterance.onend = () => { isSpeaking = false; };
      utterance.onerror = () => { isSpeaking = false; };
      
      window.speechSynthesis.speak(utterance);
    };
  });

  // 슬라이더를 조작할 때마다 설정을 저장하는 함수
  function saveSettings() {
    localStorage.setItem('biophys_audio_settings', JSON.stringify({ isMuted, volume, pitch, rate }));
  }
</script>

<div class="relative w-full max-w-4xl mx-auto my-4 font-sans">
  <!-- 메인 오디오 컨트롤 바 -->
  <div class="flex items-center justify-between p-4 bg-black/40 backdrop-blur-2xl rounded-2xl border border-white/10 shadow-[0_8px_32px_rgba(0,0,0,0.3)] z-20 relative">
    
    <!-- 1. 원터치 음소거 버튼 -->
    <button 
      on:click={() => { isMuted = !isMuted; saveSettings(); }}
      class="text-2xl opacity-70 hover:opacity-100 transition-all hover:scale-110 focus:outline-none"
      title={isMuted ? "소리 켜기" : "음소거"}
    >
      {isMuted ? '🔇' : '🔊'}
    </button>

    <!-- 2. 중앙 오디오 파형 (음소거 시 흑백 처리) -->
    <div class="flex-1 flex justify-center h-12 gap-1.5 mx-6">
      {#each Array(20) as _, i}
        <div 
          class="w-1.5 rounded-full transition-all duration-100 ease-in-out 
                 {isMuted ? 'bg-gradient-to-t from-gray-700 to-gray-500' : 'bg-gradient-to-t from-blue-600 to-cyan-300'}"
          style="
            height: {isSpeaking && !isMuted ? Math.random() * 80 + 20 : 10}%; 
            animation: {isSpeaking && !isMuted ? `waveform ${Math.random() * 0.4 + 0.3}s infinite alternate ease-in-out` : 'none'};
            animation-delay: {i * 0.05}s;
          "
        ></div>
      {/each}
    </div>

    <!-- 3. 설정 패널 토글 버튼 -->
    <button 
      on:click={() => showSettings = !showSettings}
      class="text-2xl opacity-70 hover:opacity-100 transition-all hover:scale-110 focus:outline-none {showSettings ? 'text-blue-400 opacity-100 drop-shadow-[0_0_8px_rgba(96,165,250,0.8)]' : ''}"
      title="상세 오디오 설정"
    >
      ⚙️
    </button>
  </div>

  <!-- 확장 세부 설정(Settings) 패널 (글래스모피즘 적용) -->
  {#if showSettings}
    <div class="absolute top-full left-0 w-full mt-3 bg-black/60 backdrop-blur-3xl border border-white/10 rounded-2xl p-6 shadow-[0_15px_50px_rgba(0,0,0,0.6)] z-10 flex flex-col gap-5 transition-all origin-top">
      <h3 class="font-bold text-blue-300 border-b border-white/10 pb-3 text-lg flex items-center gap-2">
        <span class="text-xl">🎛️</span> J.A.R.V.I.S 보이스 컨트롤 튜닝
      </h3>
      
      <div class="grid grid-cols-1 md:grid-cols-3 gap-8 mt-2">
        <!-- 볼륨 슬라이더 -->
        <div class="flex flex-col gap-3">
          <label class="text-sm font-semibold text-gray-200 flex justify-between">
            <span>볼륨 (Volume)</span>
            <span class="text-blue-400">{Math.round(volume * 100)}%</span>
          </label>
          <input type="range" min="0" max="1" step="0.05" bind:value={volume} on:change={saveSettings} class="accent-blue-500 cursor-pointer" />
        </div>

        <!-- 톤(피치) 슬라이더 -->
        <div class="flex flex-col gap-3">
          <label class="text-sm font-semibold text-gray-200 flex justify-between">
            <span>목소리 톤 (Pitch)</span>
            <span class="text-blue-400">{pitch}</span>
          </label>
          <input type="range" min="0" max="2" step="0.1" bind:value={pitch} on:change={saveSettings} class="accent-cyan-500 cursor-pointer" />
          <p class="text-xs text-gray-500">낮을수록 진중하고, 높을수록 통통 튑니다.</p>
        </div>

        <!-- 속도 슬라이더 -->
        <div class="flex flex-col gap-3">
          <label class="text-sm font-semibold text-gray-200 flex justify-between">
            <span>말하기 속도 (Rate)</span>
            <span class="text-blue-400">{rate}x</span>
          </label>
          <input type="range" min="0.5" max="2" step="0.05" bind:value={rate} on:change={saveSettings} class="accent-indigo-500 cursor-pointer" />
          <p class="text-xs text-gray-500">1.15x 이상을 권장합니다.</p>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  @keyframes waveform {
    0% { height: 15%; opacity: 0.6; }
    100% { height: 100%; opacity: 1; box-shadow: 0 0 15px rgba(56, 189, 248, 0.8); }
  }
</style>
