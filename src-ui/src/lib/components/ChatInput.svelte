<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();
  let message = '';
  let isDragging = false;
  let attachedImage: string | null = null;

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    isDragging = true;
  }
  
  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    isDragging = false;
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    isDragging = false;
    
    const files = e.dataTransfer?.files;
    if (files && files.length > 0) {
      const file = files[0];
      // 이미지만 허용
      if (file.type.startsWith('image/')) {
        // Tauri 샌드박스 우회를 위해 로컬 경로 대신 인메모리 Base64로 즉시 변환
        const reader = new FileReader();
        reader.onload = (event) => {
          attachedImage = event.target?.result as string;
        };
        reader.readAsDataURL(file);
      }
    }
  }

  function removeImage() {
    attachedImage = null;
  }

  function submit() {
    if (!message && !attachedImage) return;
    dispatch('send', { text: message, image: attachedImage });
    message = '';
    attachedImage = null;
  }
</script>

<!-- Tauri 앱 윈도우 전체에서 드래그 이벤트가 무시되거나 파일이 열리는 현상 원천 차단 -->
<svelte:window 
  on:dragover|preventDefault 
  on:drop|preventDefault 
/>

<!-- Liquid Glass 다크모드 기반 채팅 입력창 -->
<div 
  class="relative flex flex-col w-full max-w-4xl mx-auto rounded-3xl border transition-all duration-300 {isDragging ? 'border-blue-400 bg-white/10 scale-[1.02]' : 'border-white/10 bg-black/40'} backdrop-blur-2xl shadow-[0_8px_32px_0_rgba(0,0,0,0.5)] p-5"
  on:dragover={handleDragOver}
  on:dragleave={handleDragLeave}
  on:drop={handleDrop}
  role="region"
  aria-label="채팅 입력 영역"
>
  <!-- 드래그 오버레이 (사진을 끌어왔을 때 뜨는 안내창) -->
  {#if isDragging}
    <div class="absolute inset-0 z-10 flex items-center justify-center bg-blue-500/20 rounded-3xl backdrop-blur-md border-2 border-dashed border-blue-400 transition-all">
      <span class="text-blue-100 font-bold text-xl drop-shadow-lg">📸 사진을 이곳에 끌어다 놓으세요</span>
    </div>
  {/if}

  <!-- 첨부된 사진 썸네일 미리보기 -->
  {#if attachedImage}
    <div class="relative w-28 h-28 mb-4 rounded-2xl overflow-hidden border border-white/20 shadow-xl group transition-all">
      <img src={attachedImage} alt="첨부된 사진" class="w-full h-full object-cover" />
      <!-- 삭제 버튼 -->
      <button 
        class="absolute top-2 right-2 bg-black/70 hover:bg-red-500/90 text-white rounded-full w-7 h-7 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-all backdrop-blur-sm"
        on:click={removeImage}
        aria-label="사진 삭제"
      >
        ✕
      </button>
    </div>
  {/if}

  <!-- 입력 필드 및 전송 버튼 -->
  <div class="flex items-center gap-4">
    <input 
      type="text" 
      bind:value={message}
      placeholder="메시지를 입력하거나 바탕화면의 사진을 끌어다 놓으세요..."
      class="flex-1 bg-transparent text-gray-100 placeholder-gray-500 focus:outline-none px-2 text-lg"
      on:keydown={(e) => e.key === 'Enter' && submit()}
    />
    <button 
      on:click={submit}
      class="bg-white/10 hover:bg-blue-600/80 text-white px-6 py-3 rounded-2xl border border-white/10 hover:border-blue-400/50 backdrop-blur-lg shadow-[0_4px_20px_rgba(37,99,235,0.2)] transition-all font-semibold"
    >
      전송
    </button>
  </div>
</div>
