<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  export let dangerousAction = "";
  export let show = false;
  
  const dispatch = createEventDispatcher();
  
  function approve() {
    dispatch('approve');
    show = false;
  }
  function reject() {
    dispatch('reject');
    show = false;
  }
</script>

{#if show}
  <!-- Full Screen Blur Overlay -->
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xl">
    
    <!-- Cybernetic Red Modal -->
    <div class="relative liquid-glass !bg-red-950/40 !border-red-500/30 rounded-3xl p-1 max-w-xl w-full shadow-[0_0_100px_rgba(220,38,38,0.2)] transform transition-all scale-105 overflow-hidden">
      
      <!-- Scanline effect overlay -->
      <div class="absolute inset-0 pointer-events-none bg-[linear-gradient(rgba(220,38,38,0.03)_50%,transparent_50%)] bg-[length:100%_4px] z-0"></div>

      <div class="relative z-10 p-8 sm:p-10 flex flex-col gap-6">
        
        <!-- Header -->
        <div class="flex items-center gap-5 border-b border-red-500/20 pb-6">
          <div class="relative flex items-center justify-center w-14 h-14 rounded-full bg-red-900/50 border border-red-500/50">
            <span class="text-red-400 text-2xl animate-pulse">⚠️</span>
            <div class="absolute inset-0 rounded-full border border-red-500 animate-ping"></div>
          </div>
          <div>
            <h2 class="text-2xl font-black text-red-400 tracking-[0.2em] drop-shadow-[0_0_10px_rgba(248,113,113,0.5)]">SECURITY OVERRIDE</h2>
            <p class="text-xs text-red-500/70 font-mono tracking-widest mt-1 uppercase">Intent Gateway Active</p>
          </div>
        </div>
        
        <div class="py-2">
          <p class="text-red-200 mb-3 font-mono text-sm leading-relaxed">
            경고: 에이전트가 허가되지 않은 <span class="font-bold text-red-400 bg-red-900/40 px-1 rounded">치명적인 시스템 제어</span>를 시도했습니다. 
            위상 절연체 장벽이 해당 명령을 샌드박스에 억류했습니다.
          </p>
          
          <div class="bg-black/60 p-5 rounded-xl text-red-400 font-mono text-xs border border-red-500/20 shadow-inner break-all">
            <span class="text-red-600 mr-2">root@biophys:~#</span> {dangerousAction}
          </div>
        </div>
        
        <!-- Action Buttons -->
        <div class="flex gap-4 mt-2">
          <button on:click={reject} class="flex-1 liquid-glass !bg-slate-900/50 hover:!bg-slate-800/80 !border-white/5 text-slate-300 py-4 rounded-2xl uppercase font-mono text-sm font-bold tracking-widest transition-all hover:scale-[0.98]">
            차단 (Block)
          </button>
          <button on:click={approve} class="flex-1 bg-gradient-to-r from-red-700 to-red-600 hover:from-red-600 hover:to-red-500 text-white py-4 rounded-2xl uppercase font-mono text-sm font-bold tracking-widest transition-all hover:scale-[0.98] border border-red-400/50 shadow-[0_0_20px_rgba(220,38,38,0.4)]">
            물리적 승인
          </button>
        </div>

      </div>
    </div>
  </div>
{/if}
