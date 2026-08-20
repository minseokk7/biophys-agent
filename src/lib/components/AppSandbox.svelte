<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { createEventDispatcher } from "svelte";

  export let appId = "";
  export let appName = "자율 생성 앱";
  export let appType = "svelte";
  export let bundleHtml = "";
  export let isOpen = false;

  const dispatch = createEventDispatcher();

  let iframeEl: HTMLIFrameElement;
  let isExporting = false;
  let exportSuccessPath = "";

  function reloadSandbox() {
    if (iframeEl) {
      const current = iframeEl.srcdoc;
      iframeEl.srcdoc = '';
      setTimeout(() => {
        iframeEl.srcdoc = current;
      }, 50);
    }
  }

  async function handleExport() {
    if (!appId) return;
    isExporting = true;
    exportSuccessPath = "";
    try {
      const path = await invoke("export_generated_app", { id: appId, customDestDir: null }) as string;
      exportSuccessPath = path;
    } catch (e) {
      alert("내보내기 실패: " + e);
    }
    isExporting = false;
  }

  function closeSandbox() {
    isOpen = false;
    dispatch("close");
  }
  // HTML 조각을 완전한 독립형 다크 글래스모피즘 웹 애플리케이션으로 래핑
  $: processedHtml = (() => {
    if (!bundleHtml) return '';
    if (bundleHtml.includes('<!DOCTYPE html>')) return bundleHtml;

    return `<!DOCTYPE html>
<html lang="ko" class="dark">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${appName}</title>
  <script src="https://cdn.tailwindcss.com"><\/script>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link href="https://fonts.googleapis.com/css2?family=Pretendard:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;600&display=swap" rel="stylesheet">
  <style>
    * {
      box-sizing: border-box;
      margin: 0;
      padding: 0;
      font-family: 'Pretendard', -apple-system, BlinkMacSystemFont, system-ui, Roboto, sans-serif;
    }
    body {
      background: radial-gradient(circle at 50% 0%, #0c1527 0%, #030712 100%);
      min-height: 100vh;
      color: #f8fafc;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      padding: 1.25rem;
    }
    .glass-panel {
      background: rgba(15, 23, 42, 0.75);
      backdrop-filter: blur(20px);
      -webkit-backdrop-filter: blur(20px);
      border: 1px solid rgba(255, 255, 255, 0.12);
      border-radius: 1.5rem;
      box-shadow: 0 20px 40px -15px rgba(0, 0, 0, 0.7), 0 0 30px rgba(6, 182, 212, 0.1);
      width: 100%;
    }
    input {
      color: #ffffff !important;
      background: rgba(2, 6, 23, 0.6) !important;
      border: 1px solid rgba(255, 255, 255, 0.15) !important;
    }
    input:focus {
      outline: none !important;
      border-color: #22d3ee !important;
      box-shadow: 0 0 10px rgba(34, 211, 238, 0.3) !important;
    }
    button {
      cursor: pointer;
      font-family: inherit;
    }
  </style>
</head>
<body class="selection:bg-cyan-500 selection:text-black">
  <div class="w-full">
    ${bundleHtml}
  </div>
</body>
</html>`;
  })();
</script>

{#if isOpen && processedHtml}
  <div class="fixed inset-y-6 right-6 w-full max-w-[460px] z-50 flex flex-col rounded-[2rem] bg-slate-950/90 backdrop-blur-2xl border border-white/15 shadow-[0_25px_60px_-15px_rgba(0,0,0,0.8),0_0_40px_rgba(6,182,212,0.15)] ring-1 ring-cyan-500/30 overflow-hidden animate-fade-in-up">
    
    <!-- TOP CONTROL BAR -->
    <div class="h-14 px-5 flex items-center justify-between border-b border-white/10 bg-black/40 shrink-0">
      <div class="flex items-center gap-3">
        <div class="w-3 h-3 rounded-full bg-cyan-400 animate-pulse shadow-[0_0_8px_#22d3ee]"></div>
        <div class="flex flex-col">
          <span class="text-sm font-bold text-white tracking-wide">{appName}</span>
          <span class="text-[10px] font-mono text-cyan-300 uppercase tracking-widest">Live Sandbox Mode</span>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <!-- RELOAD BUTTON -->
        <button 
          on:click={reloadSandbox} 
          title="샌드박스 다시 로드"
          class="w-8 h-8 rounded-xl flex items-center justify-center bg-white/5 hover:bg-white/15 text-slate-300 hover:text-white transition-all active:scale-95"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path></svg>
        </button>

        <!-- EXPORT STANDALONE BUTTON -->
        <button 
          on:click={handleExport}
          disabled={isExporting}
          title="바탕화면에 독립 실행 파일로 내보내기"
          class="px-3 py-1.5 rounded-xl flex items-center gap-1.5 bg-cyan-500/20 hover:bg-cyan-500/30 text-cyan-300 border border-cyan-500/40 text-xs font-semibold font-mono transition-all active:scale-95 shadow-[0_0_10px_rgba(6,182,212,0.2)]"
        >
          {#if isExporting}
            <span class="animate-spin text-[10px]">⏳</span>
          {:else}
            <span>📦</span>
          {/if}
          <span>내보내기</span>
        </button>

        <!-- CLOSE BUTTON -->
        <button 
          on:click={closeSandbox}
          class="w-8 h-8 rounded-xl flex items-center justify-center bg-red-500/10 hover:bg-red-500/30 text-red-400 hover:text-red-200 transition-all active:scale-95"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
        </button>
      </div>
    </div>

    <!-- EXPORT NOTIFICATION -->
    {#if exportSuccessPath}
      <div class="bg-emerald-500/10 border-b border-emerald-500/30 px-4 py-2 flex items-center justify-between text-xs text-emerald-300 font-mono">
        <span class="truncate">🎉 바탕화면에 내보내기 완료!</span>
        <button on:click={() => exportSuccessPath = ''} class="underline text-[10px] ml-2 text-emerald-400">닫기</button>
      </div>
    {/if}

    <!-- LIVE IFRAME VIEWPORT -->
    <div class="flex-1 w-full h-full bg-[#020617] relative">
      <iframe
        bind:this={iframeEl}
        title={appName}
        srcdoc={processedHtml}
        class="w-full h-full border-none rounded-b-[2rem]"
        sandbox="allow-scripts allow-same-origin allow-modals allow-forms"
      ></iframe>
    </div>
  </div>
{/if}

<style>
  @keyframes fadeInUp {
    from { opacity: 0; transform: translateX(20px) scale(0.98); }
    to { opacity: 1; transform: translateX(0) scale(1); }
  }
  .animate-fade-in-up {
    animation: fadeInUp 0.4s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }
</style>
