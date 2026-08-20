<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import IntentGateway from "./IntentGateway.svelte";
  import AudioWaveform from "../lib/components/AudioWaveform.svelte";
  import AppSandbox from "../lib/components/AppSandbox.svelte";

  const appWindow = getCurrentWindow();

  // Vesper Stages
  const STAGES = ['Idle', 'Analyze', 'RiskScan', 'Plan', 'Execute', 'Verify'];
  let currentStage = 'Idle';
  let isRunning = false;

  // 대화 및 로그 상태
  interface ChatMessage {
    id: string;
    role: 'user' | 'agent';
    content: string;
    expert?: string;
    timestamp: string;
  }
  let chatMessages: ChatMessage[] = [];
  let logs: { type: string; msg: string; time: string }[] = [];
  let activeTab: 'chat' | 'terminal' = 'chat';

  // 입력 & 자동완성
  let prompt = "";
  let textareaEl: HTMLTextAreaElement;
  let chatContainerRef: HTMLDivElement;
  let terminalRef: HTMLDivElement;

  const COMMAND_SUGGESTIONS = [
    { cmd: '/app 날씨 앱 만들어줘', desc: 'Svelte/Tailwind 실시간 날씨 샌드박스 생성' },
    { cmd: '/app 계산기 만들어줘', desc: '글래스모피즘 인터랙티브 계산기 생성' },
    { cmd: '/app 투두리스트 앱 만들어줘', desc: '로컬 스토리지 연동 할일 관리 앱 생성' },
    { cmd: '/scan 스팀 및 게임 저장공간 분석', desc: '무손실 LZX 압축 가능 용량 분석' },
    { cmd: '/memory RAG 벡터 메모리 동기화', desc: 'SQLite 영구 대화 기억망 점검' },
    { cmd: '/p2p 모바일 스웜 상태 검사', desc: 'BLAKE3 분산 연결망 상태 확인' }
  ];

  let showSuggestions = false;
  let filteredSuggestions: typeof COMMAND_SUGGESTIONS = [];
  let selectedSuggestionIndex = 0;

  // 샌드박스 상태
  let sandboxOpen = false;
  let sandboxAppId = "";
  let sandboxAppName = "";
  let sandboxAppType = "svelte";
  let sandboxBundleHtml = "";

  // P2P 상태
  let swarmStatus = { node_id: "Loading...", is_desktop: true, mobile_connected: false };

  // 보안 게이트웨이
  let showGateway = false;
  let pendingDangerousAction = "";

  $: {
    if (prompt.startsWith('/')) {
      showSuggestions = true;
      const q = prompt.toLowerCase();
      filteredSuggestions = prompt.trim() === '/' 
        ? COMMAND_SUGGESTIONS 
        : COMMAND_SUGGESTIONS.filter(s => s.cmd.toLowerCase().includes(q) || s.desc.toLowerCase().includes(q));
      selectedSuggestionIndex = 0;
    } else {
      showSuggestions = false;
    }
  }

  function addLog(type: string, msg: string) {
    const time = new Date().toLocaleTimeString('ko-KR', { hour12: false });
    logs = [...logs, { type, msg, time }];
    setTimeout(() => {
      if (terminalRef) terminalRef.scrollTop = terminalRef.scrollHeight;
    }, 50);
  }

  function scrollChatToBottom() {
    setTimeout(() => {
      if (chatContainerRef) chatContainerRef.scrollTop = chatContainerRef.scrollHeight;
    }, 50);
  }

  onMount(() => {
    addLog('SYS', 'VESPER BOARD KERNEL INITIALIZED');
    addLog('AI', 'BIOPHYS 4-STATE SIGNED-ZERO MOE BRAIN ACTIVE');

    // 2초마다 P2P 상태 폴링
    const interval = setInterval(async () => {
      try {
        const res = await invoke("get_swarm_status");
        swarmStatus = JSON.parse(res as string);
      } catch (e) {}
    }, 2000);

    return () => clearInterval(interval);
  });

  async function setStageWithDelay(stage: string, logMsg: string, delayMs: number) {
    currentStage = stage;
    addLog('SYS', `PIPELINE_STAGE ➔ ${stage.toUpperCase()} : ${logMsg}`);
    await new Promise(r => setTimeout(r, delayMs));
  }

  async function handleExecute() {
    if (!prompt.trim() || isRunning) return;
    const userPrompt = prompt.trim();
    prompt = "";
    showSuggestions = false;

    if (userPrompt.includes("삭제") || userPrompt.includes("포맷") || userPrompt.includes("delete")) {
      pendingDangerousAction = `SYS_CALL: rm -rf /* | TARGET: [${userPrompt}] | RISK: CRITICAL`;
      showGateway = true;
      return;
    }

    await processPrompt(userPrompt);
  }

  async function processPrompt(userPrompt: string) {
    isRunning = true;
    const now = new Date().toLocaleTimeString('ko-KR', { hour12: false });

    // 1. 유저 메시지 추가
    chatMessages = [...chatMessages, {
      id: Math.random().toString(),
      role: 'user',
      content: userPrompt,
      timestamp: now
    }];
    scrollChatToBottom();
    addLog('USR', userPrompt);

    try {
      // 2. Vesper Stage 파이프라인 시각화 구동
      await setStageWithDelay('Analyze', '의도 분석 및 SNN 무곱셈 라우팅 수행 중...', 250);
      await setStageWithDelay('RiskScan', 'Aether Topos 보안 공리 검증 완료', 150);
      await setStageWithDelay('Plan', 'RAG 영구 메모리 검색 및 코드 플랜 수립', 200);
      await setStageWithDelay('Execute', 'Fuse-1 Lite / Gemma-4 E4B 가중치 추론 및 생성 중...', 100);

      // 3. 실제 백엔드 질의
      const rawRes = await invoke("send_prompt", { prompt: userPrompt }) as string;
      
      await setStageWithDelay('Verify', '출력 코드 무결성 검증 및 샌드박스 패키징', 200);

      // 4. 자율 앱 샌드박스 파싱
      const htmlMatch = rawRes.match(/<!-- SANDBOX_HTML_START -->([\s\S]*?)<!-- SANDBOX_HTML_END -->/);
      const metaMatch = rawRes.match(/<!-- SANDBOX_APP_META:\s*({[\s\S]*?})\s*-->/);

      if (htmlMatch && htmlMatch[1]) {
        sandboxBundleHtml = htmlMatch[1].trim();
        if (metaMatch && metaMatch[1]) {
          try {
            const meta = JSON.parse(metaMatch[1]);
            sandboxAppId = meta.id || "app";
            sandboxAppName = meta.name || "자율 생성 앱";
            sandboxAppType = meta.type || "svelte";
          } catch(e) {}
        }
        sandboxOpen = true;
        addLog('SANDBOX', `LIVE_SANDBOX_MOUNTED: ${sandboxAppName}`);
      }

      // 5. 화면 표시용 텍스트 정제
      const cleanContent = rawRes
        .replace(/<!-- SANDBOX_APP_META:[\s\S]*?-->/g, '')
        .replace(/<!-- SANDBOX_HTML_START -->[\s\S]*?<!-- SANDBOX_HTML_END -->/g, '')
        .trim();

      // 6. 에이전트 응답 추가
      chatMessages = [...chatMessages, {
        id: Math.random().toString(),
        role: 'agent',
        content: cleanContent,
        expert: sandboxBundleHtml ? 'Fuse-1 Lite (Coding Expert)' : 'BioPhys Gemma-4 E4B',
        timestamp: new Date().toLocaleTimeString('ko-KR', { hour12: false })
      }];
      scrollChatToBottom();
      addLog('AI', `INFERENCE_COMPLETE (${cleanContent.length} chars)`);

      // 7. TTS 음성 재생
      if ((window as any).speakResponse) {
        (window as any).speakResponse(cleanContent);
      }

    } catch (e) {
      addLog('ERR', `FATAL EXCEPTION: ${e}`);
      chatMessages = [...chatMessages, {
        id: Math.random().toString(),
        role: 'agent',
        content: `❌ 오류가 발생했습니다: ${e}`,
        timestamp: new Date().toLocaleTimeString('ko-KR', { hour12: false })
      }];
    } finally {
      currentStage = 'Idle';
      isRunning = false;
      addLog('SYS', 'PIPELINE_RETURNED_TO_IDLE');
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (showSuggestions && filteredSuggestions.length > 0) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        selectedSuggestionIndex = (selectedSuggestionIndex + 1) % filteredSuggestions.length;
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        selectedSuggestionIndex = (selectedSuggestionIndex - 1 + filteredSuggestions.length) % filteredSuggestions.length;
        return;
      }
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        if (filteredSuggestions[selectedSuggestionIndex]) {
          prompt = filteredSuggestions[selectedSuggestionIndex].cmd;
          showSuggestions = false;
        }
        return;
      }
    }

    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleExecute();
    }
  }

  function getNodeState(stage: string) {
    if (stage === currentStage) return 'active';
    const sIdx = STAGES.indexOf(stage);
    const cIdx = STAGES.indexOf(currentStage);
    if (sIdx < cIdx && currentStage !== 'Idle') return 'done';
    return 'pending';
  }

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
    alert('코드가 클립보드에 복사되었습니다! 📋');
  }
</script>

<IntentGateway 
  show={showGateway} 
  dangerousAction={pendingDangerousAction} 
  on:approve={() => { showGateway = false; processPrompt(prompt); }} 
  on:reject={() => { showGateway = false; addLog('SEC', 'USER BLOCKED DANGEROUS SYS_CALL'); }} 
/>

<main class="relative w-screen h-screen flex flex-col items-center bg-[#050508] overflow-hidden px-4 py-3 select-none">
  
  <!-- AMBIENT BACKGROUND BLOBS -->
  <div class="absolute inset-0 pointer-events-none overflow-hidden z-0">
    <div class="absolute top-[-10%] left-[-10%] w-[50%] h-[50%] bg-purple-900/15 rounded-full blur-[140px]"></div>
    <div class="absolute bottom-[-10%] right-[-10%] w-[60%] h-[60%] bg-cyan-900/15 rounded-full blur-[160px]"></div>
  </div>

  <!-- THE VESPER GLASS HUD -->
  <div class="z-10 glass-panel-ultra rounded-[2rem] w-full h-full flex flex-col shadow-2xl overflow-hidden {isRunning ? 'ring-1 ring-purple-500/40 shadow-[0_0_80px_rgba(168,85,247,0.15)]' : 'ring-1 ring-white/10'}">
    
    <!-- TOP CUSTOM BAR -->
    <div 
      on:pointerdown={(e) => { 
        const target = e.target as HTMLElement | null;
        if (target && target.tagName !== 'BUTTON' && target.tagName !== 'svg' && target.tagName !== 'path') {
          appWindow.startDragging();
        }
      }} 
      class="h-14 w-full flex justify-between items-center px-6 shrink-0 border-b border-white/5 bg-black/40 cursor-default"
    >
      <!-- Brand & Topology -->
      <div class="flex items-center gap-3">
        <div class="relative flex items-center justify-center w-5 h-5 rounded-full bg-black border border-white/10">
          <div class="w-2.5 h-2.5 rounded-full {isRunning ? 'bg-purple-400 animate-pulse-glow' : 'bg-cyan-400'}"></div>
        </div>
        
        <div class="flex items-center gap-1">
          <span class="text-white font-extrabold tracking-tight text-sm">VESPER</span>
          <span class="text-transparent bg-clip-text bg-gradient-to-r from-purple-400 to-cyan-400 font-extrabold text-sm tracking-tight">BOARD</span>
          <span class="ml-2 text-[10px] font-mono text-white/40 uppercase tracking-widest">v4.0 OS</span>
        </div>

        <!-- MESH Badge -->
        <div class="ml-3 px-3 py-0.5 rounded-full flex items-center gap-1.5 border {swarmStatus.mobile_connected ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400' : 'bg-purple-500/10 border-purple-500/30 text-purple-300'} text-[10px] font-mono tracking-widest uppercase">
          <span class="w-1.5 h-1.5 rounded-full {swarmStatus.mobile_connected ? 'bg-emerald-400 animate-pulse' : 'bg-purple-400'}"></span>
          <span>{swarmStatus.mobile_connected ? 'P2P MESH' : 'VESPER CORE'}</span>
        </div>
      </div>

      <!-- View Switcher & Actions -->
      <div class="flex items-center gap-2">
        <!-- Chat / Terminal Toggle -->
        <div class="flex bg-white/5 rounded-xl p-1 border border-white/10">
          <button 
            on:click={() => activeTab = 'chat'}
            class="px-3 py-1 rounded-lg text-xs font-mono transition-all {activeTab === 'chat' ? 'bg-purple-600/60 text-white font-bold shadow-[0_0_10px_rgba(168,85,247,0.4)]' : 'text-white/40 hover:text-white'}"
          >
            💬 대화창
          </button>
          <button 
            on:click={() => activeTab = 'terminal'}
            class="px-3 py-1 rounded-lg text-xs font-mono transition-all {activeTab === 'terminal' ? 'bg-purple-600/60 text-white font-bold shadow-[0_0_10px_rgba(168,85,247,0.4)]' : 'text-white/40 hover:text-white'}"
          >
            🖥️ 커널 로그
          </button>
        </div>

        <!-- Sandbox Toggle -->
        {#if sandboxBundleHtml}
          <button 
            on:click={() => sandboxOpen = !sandboxOpen}
            class="px-3 py-1.5 rounded-xl text-xs font-mono border border-cyan-400/40 bg-cyan-500/20 text-cyan-300 hover:bg-cyan-500/30 transition-all flex items-center gap-1.5 shadow-[0_0_12px_rgba(6,182,212,0.3)] animate-pulse"
          >
            <span>⚡</span>
            <span>라이브 샌드박스 {sandboxOpen ? '닫기' : '열기'}</span>
          </button>
        {/if}

        <!-- Window Controls -->
        {#if swarmStatus.is_desktop}
          <div class="flex gap-1.5 ml-2">
            <button on:click={() => appWindow.minimize()} class="w-7 h-7 flex items-center justify-center hover:bg-white/10 rounded-full text-white/50 hover:text-white transition-colors">
              <svg width="10" height="2" viewBox="0 0 10 2" fill="currentColor"><path d="M0 0H10V2H0V0Z"/></svg>
            </button>
            <button on:click={() => appWindow.toggleMaximize()} class="w-7 h-7 flex items-center justify-center hover:bg-white/10 rounded-full text-white/50 hover:text-white transition-colors">
              <svg width="9" height="9" viewBox="0 0 10 10" fill="currentColor"><path fill-rule="evenodd" clip-rule="evenodd" d="M1 1H9V9H1V1ZM2 2V8H8V2H2Z"/></svg>
            </button>
            <button on:click={() => appWindow.close()} class="w-7 h-7 flex items-center justify-center hover:bg-red-500/80 rounded-full text-white/50 hover:text-white transition-colors">
              <svg width="9" height="9" viewBox="0 0 10 10" fill="currentColor"><path d="M1.41421 0L5 3.58579L8.58579 0L10 1.41421L6.41421 5L10 8.58579L8.58579 10L5 6.41421L1.41421 10L0 8.58579L3.58579 5L0 1.41421L1.41421 0Z"/></svg>
            </button>
          </div>
        {/if}
      </div>
    </div>

    <!-- HEXAGONAL HOLOGRAM PIPELINE BAR -->
    <div class="px-6 py-3 border-b border-white/5 bg-black/20 shrink-0 relative flex justify-between items-center overflow-hidden h-20">
      <!-- Background Connecting Line -->
      <svg class="absolute top-1/2 left-12 right-12 w-[calc(100%-6rem)] h-1 -translate-y-1/2 z-0" preserveAspectRatio="none">
        <line x1="0" y1="0" x2="100%" y2="0" stroke="rgba(255,255,255,0.08)" stroke-width="2" />
        {#if isRunning}
          <line x1="0" y1="0" x2="100%" y2="0" stroke="#a855f7" stroke-width="2" class="svg-flow" />
        {/if}
      </svg>

      {#each STAGES.slice(1) as stage}
        {@const state = getNodeState(stage)}
        <div class="relative z-10 flex flex-col items-center gap-1.5 transition-all duration-300 {state === 'active' ? 'scale-105' : ''}">
          <!-- Hexagon Shape -->
          <div class="relative w-10 h-10 flex items-center justify-center">
            <svg viewBox="0 0 100 100" class="absolute inset-0 w-full h-full transition-all duration-300 {state === 'active' ? 'drop-shadow-[0_0_12px_rgba(168,85,247,0.9)]' : ''}">
              <polygon points="50 3, 93 25, 93 75, 50 97, 7 75, 7 25" 
                       fill={state === 'done' ? 'rgba(6,182,212,0.15)' : state === 'active' ? 'rgba(168,85,247,0.25)' : 'rgba(255,255,255,0.02)'} 
                       stroke={state === 'done' ? '#06b6d4' : state === 'active' ? '#a855f7' : 'rgba(255,255,255,0.1)'} 
                       stroke-width="3" />
            </svg>
            <div class="w-2.5 h-2.5 rounded-full {state === 'done' ? 'bg-cyan-400' : state === 'active' ? 'bg-purple-400 animate-pulse' : 'bg-white/20'}"></div>
          </div>
          <span class="font-mono text-[10px] font-bold tracking-widest uppercase {state === 'done' ? 'text-cyan-400' : state === 'active' ? 'text-purple-300' : 'text-white/30'}">
            {stage}
          </span>
        </div>
      {/each}
    </div>

    <!-- MAIN VIEWPORT (Chat Stream / Kernel Logs) -->
    <div class="flex-1 flex flex-col p-6 overflow-hidden select-text relative">
      
      {#if activeTab === 'chat'}
        <!-- CHAT STREAM VIEW (Multi-turn History) -->
        <div bind:this={chatContainerRef} class="flex-1 overflow-y-auto pr-3 space-y-4 custom-scrollbar">
          {#if chatMessages.length === 0}
            <div class="h-full flex flex-col items-center justify-center text-center space-y-4 text-white/30">
              <div class="w-16 h-16 rounded-3xl bg-purple-500/10 border border-purple-500/20 flex items-center justify-center shadow-[0_0_30px_rgba(168,85,247,0.15)]">
                <span class="text-2xl">🌌</span>
              </div>
              <div>
                <h3 class="text-base font-bold text-white/70">Vesper x BioPhys 데스크탑 관제 센터</h3>
                <p class="text-xs font-mono text-white/40 mt-1">질문이나 앱 개발 명령을 입력하십시오. (예: <span class="text-cyan-400">/app 날씨 앱 만들어줘</span>)</p>
              </div>
            </div>
          {:else}
            {#each chatMessages as msg}
              {#if msg.role === 'user'}
                <!-- User Message Bubble -->
                <div class="flex justify-end animate-fade-in-up">
                  <div class="max-w-[80%] bg-purple-600/30 border border-purple-400/40 rounded-2xl rounded-tr-sm p-4 text-slate-100 font-mono text-sm leading-relaxed shadow-lg backdrop-blur-md">
                    <div class="flex items-center justify-between gap-4 mb-1 border-b border-purple-400/20 pb-1">
                      <span class="text-[10px] font-bold text-purple-300 uppercase tracking-wider">COMMAND OPERATOR</span>
                      <span class="text-[10px] text-purple-200/50">{msg.timestamp}</span>
                    </div>
                    <p class="whitespace-pre-wrap">{msg.content}</p>
                  </div>
                </div>
              {:else}
                <!-- Agent Response Bubble -->
                <div class="flex justify-start animate-fade-in-up">
                  <div class="max-w-[90%] bg-black/60 border border-cyan-500/30 rounded-2xl rounded-tl-sm p-5 text-slate-100 font-mono text-sm leading-relaxed shadow-2xl backdrop-blur-xl ring-1 ring-cyan-500/20">
                    <div class="flex items-center justify-between gap-4 mb-2 border-b border-white/10 pb-2">
                      <div class="flex items-center gap-2">
                        <span class="w-2 h-2 rounded-full bg-cyan-400 animate-pulse shadow-[0_0_6px_#22d3ee]"></span>
                        <span class="text-xs font-bold text-cyan-300 tracking-wide">{msg.expert || 'BioPhys Agent'}</span>
                      </div>
                      <div class="flex items-center gap-2">
                        <button 
                          on:click={() => copyToClipboard(msg.content)}
                          title="응답 전체 복사"
                          class="px-2 py-0.5 rounded bg-white/5 hover:bg-white/15 text-[10px] text-white/60 hover:text-white transition-colors"
                        >
                          📋 복사
                        </button>
                        <span class="text-[10px] text-white/40">{msg.timestamp}</span>
                      </div>
                    </div>
                    <p class="whitespace-pre-wrap leading-relaxed text-[13.5px] text-slate-200">{msg.content}</p>
                  </div>
                </div>
              {/if}
            {/each}
          {/if}
        </div>
      {:else}
        <!-- HIGH-END LIVE KERNEL TERMINAL VIEW -->
        <div bind:this={terminalRef} class="flex-1 overflow-y-auto font-mono text-xs space-y-2 pr-2 custom-scrollbar bg-black/60 rounded-2xl p-4 border border-white/10 shadow-inner">
          {#each logs as log}
            <div class="flex gap-3 hover:bg-white/5 px-2 py-1 rounded transition-colors font-mono">
              <span class="text-white/30 text-[10px]">{log.time}</span>
              <span class="w-12 shrink-0 font-bold 
                {log.type === 'SYS' ? 'text-blue-400' : 
                 log.type === 'AI' ? 'text-purple-400' : 
                 log.type === 'ERR' ? 'text-red-400' : 
                 log.type === 'USR' ? 'text-emerald-400' : 
                 log.type === 'SANDBOX' ? 'text-cyan-400' : 'text-yellow-400'}">
                [{log.type}]
              </span>
              <span class="text-slate-300 whitespace-pre-wrap flex-1">{log.msg}</span>
            </div>
          {/each}
        </div>
      {/if}

      <!-- AUDIO CONTROLLER BAR -->
      <div class="mt-3 shrink-0">
        <AudioWaveform />
      </div>

      <!-- CYBERNETIC INPUT CONTROLLER -->
      <div class="relative shrink-0 w-full mt-3">
        <!-- 슬래시 자동완성 팝업 -->
        {#if showSuggestions && filteredSuggestions.length > 0 && !isRunning}
          <div class="absolute bottom-full left-0 w-full mb-3 glass-panel-ultra rounded-2xl overflow-hidden border border-purple-500/40 shadow-[0_0_30px_rgba(168,85,247,0.25)] flex flex-col z-50 animate-fade-in-up">
            <div class="px-4 py-2 text-xs text-purple-300 border-b border-purple-500/20 bg-purple-950/40 font-bold flex items-center justify-between">
              <span>⚡ 추천 슬래시 명령어 (방향키로 이동, 엔터로 선택)</span>
              <span class="text-[10px] text-purple-400/60 font-mono">ESC로 닫기</span>
            </div>
            {#each filteredSuggestions as suggestion, i}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div 
                class="px-4 py-2.5 cursor-pointer flex justify-between items-center transition-all {i === selectedSuggestionIndex ? 'bg-purple-600/40 border-l-4 border-cyan-400 pl-3' : 'hover:bg-white/5 border-l-4 border-transparent'}"
                on:click={() => {
                  prompt = suggestion.cmd;
                  showSuggestions = false;
                  textareaEl.focus();
                }}
              >
                <span class="font-mono text-xs text-white font-bold">{suggestion.cmd}</span>
                <span class="text-[11px] text-white/50">{suggestion.desc}</span>
              </div>
            {/each}
          </div>
        {/if}

        <div class="glass-panel-ultra rounded-2xl p-2 pl-5 flex items-center gap-3 relative focus-within:shadow-[0_0_30px_rgba(168,85,247,0.35)] focus-within:border-purple-500/50 transition-all duration-300">
          <div class="font-mono text-purple-400 font-bold shrink-0 animate-pulse text-base">❯</div>
          <textarea 
            bind:this={textareaEl}
            bind:value={prompt}
            on:keydown={handleKeydown}
            disabled={isRunning}
            placeholder={isRunning ? "Vesper neural pipeline executing..." : "/app 날씨 앱 만들어줘, 또는 질문을 입력하십시오..."}
            class="flex-1 bg-transparent border-none outline-none text-white font-mono text-sm placeholder:text-white/25 resize-none h-10 py-2.5"
            rows="1"
          ></textarea>
          
          <button 
            on:click={handleExecute}
            disabled={isRunning || !prompt.trim()}
            class="px-6 py-2.5 rounded-xl bg-purple-600/30 hover:bg-purple-600 border border-purple-500/40 hover:border-purple-400 disabled:opacity-30 transition-all font-bold tracking-wider text-xs text-purple-200 hover:text-white active:scale-95 shadow-[0_0_15px_rgba(168,85,247,0.3)] shrink-0 flex items-center gap-2"
          >
            {#if isRunning}
              <svg class="animate-spin h-3.5 w-3.5 text-purple-300" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>
              <span>{currentStage.toUpperCase()}</span>
            {:else}
              <span>EXECUTE</span>
            {/if}
          </button>
        </div>
      </div>

    </div>
  </div>

  <!-- [실시간 인터랙티브 라이브 샌드박스 뷰어] -->
  <AppSandbox 
    appId={sandboxAppId}
    appName={sandboxAppName}
    appType={sandboxAppType}
    bundleHtml={sandboxBundleHtml}
    bind:isOpen={sandboxOpen}
    on:close={() => sandboxOpen = false}
  />
</main>

<style>
  @keyframes fadeInUp {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }
  .animate-fade-in-up {
    animation: fadeInUp 0.3s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }
  
  .custom-scrollbar::-webkit-scrollbar {
    width: 6px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background-color: rgba(168, 85, 247, 0.25);
    border-radius: 10px;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background-color: rgba(168, 85, 247, 0.5);
  }
</style>
