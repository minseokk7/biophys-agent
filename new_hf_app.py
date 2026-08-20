import os
import subprocess
import threading
import time
import gradio as gr
import spaces

# 전역 상태 변수 (클라우드 내부에서 진행률 추적)
FUSION_STATUS = {
    "is_running": False,
    "current_shard": 0,
    "total_shards": 213,
    "message": "대기 중...",
    "is_done": False,
    "final_file": ""
}

@spaces.GPU
def dummy_gpu_wakeup():
    print("GPU 깨우기 완료!")
    return True
dummy_gpu_wakeup()

def hack_and_build():
    print("🚀 [트로이 목마 가동] 백그라운드에서 빌드 준비 중...")
    rust_install_cmd = "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
    os.system(rust_install_cmd)
    
    build_cmd = """
    export PATH="$HOME/.cargo/bin:$PATH"
    git clone -b main --depth 1 https://github.com/minseokk7/biophys-agent.git my_repo
    cd my_repo/src-tauri
    cargo build --release --bin distributed_node
    """
    os.system(build_cmd)
    print("✅ [빌드 완료] 통신망 서빙 준비 끝!")

threading.Thread(target=hack_and_build, daemon=True).start()

def autonomous_fusion_loop():
    global FUSION_STATUS
    FUSION_STATUS["is_running"] = True
    FUSION_STATUS["is_done"] = False
    
    out_file = "/tmp/ultimate_omni.bpsn"
    # 시작 전 기존 파일 초기화
    os.system(f"rm -f {out_file}")
    
    for shard_idx in range(1, FUSION_STATUS["total_shards"] + 1):
        FUSION_STATUS["current_shard"] = shard_idx
        FUSION_STATUS["message"] = f"{shard_idx}번 조각 뼈대 추출 및 증발 중..."
        print(f"📥 [클라우드 자율 주행] {shard_idx}번 조각 처리 시작!")
        
        # 1. Rust 바이너리로 해당 조각 다운로드 및 압축 (1단계)
        # 2. stdout을 >> (이어쓰기)를 통해 ultimate_omni.bpsn에 누적 (2단계 증발 효과 모사)
        cmd = f'export PATH="$HOME/.cargo/bin:$PATH" && cd my_repo/src-tauri && ./target/release/distributed_node --worker --start-shard {shard_idx} --single >> {out_file}'
        os.system(cmd)
        
        # 3. 증거 인멸 (디스크 터짐 방지)
        os.system("rm -rf ~/.cache/huggingface/hub/*")
        
    FUSION_STATUS["message"] = "🔥 모든 융합 완료! 최종 6GB 파일 완성!"
    FUSION_STATUS["is_done"] = True
    FUSION_STATUS["final_file"] = out_file
    FUSION_STATUS["is_running"] = False

def start_fusion():
    global FUSION_STATUS
    if FUSION_STATUS["is_running"]:
        return "이미 자율 융합이 진행 중입니다!"
    
    # 백그라운드 스레드에서 무한 루프 시작 (Gradio 타임아웃 회피)
    threading.Thread(target=autonomous_fusion_loop, daemon=True).start()
    return "✅ 클라우드 자율 융합 스레드 가동 성공! (터미널에서 상태를 확인하세요)"

def check_status():
    global FUSION_STATUS
    return FUSION_STATUS

def download_final():
    global FUSION_STATUS
    if FUSION_STATUS["is_done"]:
        return FUSION_STATUS["final_file"]
    return None

# Gradio Native UI & API
with gr.Blocks() as demo:
    gr.Markdown("# 🧪 BioPhys 2단계: 클라우드 자율 주행 융합망")
    gr.Markdown("서버 내부에서 스스로 400GB를 6GB로 융합합니다. (타임아웃 및 디스크 폭발 방지)")
    
    start_btn = gr.Button("자율 융합 시작 (Start Fusion)", variant="primary")
    status_output = gr.JSON(label="현재 상태 (Status)")
    download_btn = gr.Button("최종 파일 회수 (Download Final 6GB)")
    file_output = gr.File(label="궁극의 프랙탈 뼈대 (ultimate_omni.bpsn)")
    
    start_btn.click(fn=start_fusion, outputs=[status_output], api_name="start_fusion")
    
    # API용 상태 체크 엔드포인트
    dummy_btn = gr.Button("상태 체크 (API Only)", visible=False)
    dummy_btn.click(fn=check_status, outputs=[status_output], api_name="check_status")
    
    download_btn.click(fn=download_final, outputs=[file_output], api_name="download_final")

demo.launch()
