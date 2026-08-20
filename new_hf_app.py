import os
import subprocess
import threading
import gradio as gr
import spaces

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

def steal_shard(shard_idx: int):
    print(f"📥 {shard_idx}번 조각 압축 및 다운로드 요청 수신!")
    # 통신 병목을 뚫기 위해 이중 압축(gzip 터널링) 확장자 추가
    out_file = f"/tmp/shard_{shard_idx}.bpsn.gz"
    
    # 1. cargo run 대신 미리 빌드된 바이너리 직접 실행 (파일 락 방지)
    # 2. stdout을 gzip -1 (초고속 압축)으로 터널링하여 용량을 극한으로 줄임
    cmd = f'export PATH="$HOME/.cargo/bin:$PATH" && cd my_repo/src-tauri && ./target/release/distributed_node --worker --start-shard {shard_idx} --single | gzip -1 > {out_file}'
    os.system(cmd)
    
    # 디스크 풀(Disk Full) 에러 방지: 허깅페이스 캐시에 쌓인 원본 찌꺼기 삭제
    os.system("rm -rf ~/.cache/huggingface/hub/*")
    
    return out_file

# Gradio Native UI & API
with gr.Blocks() as demo:
    gr.Markdown("# 🧪 BioPhys 2단계: 은닉 통신망 활성화")
    gr.Markdown("서버가 백그라운드에서 데이터를 유저 컴퓨터로 쏘아보냅니다!")
    
    with gr.Row():
        shard_input = gr.Number(label="조각 번호 (Shard Index)", value=1, precision=0)
        steal_btn = gr.Button("훔쳐오기 (Steal!)", variant="primary")
        
    file_output = gr.File(label="압축된 프랙탈 뼈대 파일 (bpsn)")
    
    # 버튼 클릭 시 steal_shard 함수 실행
    steal_btn.click(fn=steal_shard, inputs=[shard_input], outputs=[file_output], api_name="steal")

demo.launch()
