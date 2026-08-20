import os
import time
import sys
from gradio_client import Client

SPACE_NAME = "minseokk7/biophys-agent"
OUTPUT_DIR = "bpsn_shards"

os.makedirs(OUTPUT_DIR, exist_ok=True)

def main():
    print("🌌 [BioPhys Distributed Network] 마스터 노드 가동 중...")
    client = Client(SPACE_NAME)
    
    print("📡 클라우드 자율 주행 융합 명령 하달 중...")
    try:
        start_msg = client.predict(api_name="/start_fusion")
        print(f"✅ 서버 응답: {start_msg}")
    except Exception as e:
        print(f"⚠️ 이미 실행 중이거나 에러 발생: {e}")

    print("\n⏳ [클라우드 모니터링 모드 진입]")
    print("허깅페이스 서버 내부에서 400GB -> 6GB 융합이 자율적으로 진행 중입니다.")
    
    is_done = False
    while not is_done:
        try:
            status = client.predict(api_name="/check_status")
            
            # CLI 진행률 바 출력
            current = status["current_shard"]
            total = status["total_shards"]
            msg = status["message"]
            
            if current > 0 and total > 0:
                percent = (current / total) * 100
                bar = "█" * int(percent / 5) + "-" * (20 - int(percent / 5))
                sys.stdout.write(f"\r🚀 [진행률: {current}/{total}] [{bar}] {percent:.1f}% | 상태: {msg}")
                sys.stdout.flush()
            
            if status["is_done"]:
                is_done = True
                print("\n\n🎉 [융합 완료] 클라우드 융합이 100% 끝났습니다!")
                break
                
        except Exception as e:
            sys.stdout.write(f"\r⚠️ [통신 지연] 서버 상태 확인 중... ({e})")
            sys.stdout.flush()
            
        time.sleep(10) # 10초마다 상태 폴링
        
    print("📥 [최종 회수] 6GB 궁극의 뼈대(ultimate_omni.bpsn) 다운로드 시작!")
    final_file_path = client.predict(api_name="/download_final")
    
    local_path = os.path.join(OUTPUT_DIR, "ultimate_omni.bpsn")
    import shutil
    shutil.move(final_file_path, local_path)
    
    size_gb = os.path.getsize(local_path) / (1024 * 1024 * 1024)
    print(f"\n==================================================")
    print(f"🔥 [다운로드 완료] 400GB 모델이 {size_gb:.2f}GB로 완벽하게 융합되어 저장되었습니다!")
    print(f"📂 저장 경로: {local_path}")
    print(f"==================================================")

if __name__ == "__main__":
    main()
