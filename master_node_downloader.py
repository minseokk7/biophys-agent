import os
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from gradio_client import Client

SPACE_NAME = "minseokk7/biophys-agent"
TOTAL_SHARDS = 213
OUTPUT_DIR = "bpsn_shards"
MAX_WORKERS = 1  # 주의: 무료 클라우드 서버는 4배속을 버티지 못하고 터집니다. 1배속으로 안전하게 훔쳐옵니다!

os.makedirs(OUTPUT_DIR, exist_ok=True)
print(f"🖥️ [Master Node] 병렬 가동 시작! 클라우드 믹서기에서 최대 {MAX_WORKERS}개의 조각을 동시에 뽑아옵니다.")

# 워커별로 독립적인 클라이언트를 생성하는 함수
def fetch_shard(shard_idx):
    print(f"📡 [요청 전송] 클라우드 믹서기에게 {shard_idx}번 파일 압축 및 전달을 지시합니다...")
    try:
        client = Client(SPACE_NAME)
        result_file_path = client.predict(shard_idx=shard_idx, api_name="/steal")
        
        # 다운로드된 파일(이중 압축 .gz)을 로컬로 가져와서 압축 해제
        local_path = os.path.join(OUTPUT_DIR, f"shard_{shard_idx}.bpsn")
        size_gz_mb = os.path.getsize(result_file_path) / (1024 * 1024)
        
        import gzip
        with gzip.open(result_file_path, "rb") as src, open(local_path, "wb") as dst:
            dst.write(src.read())
            
        size_raw_mb = os.path.getsize(local_path) / (1024 * 1024)
        os.remove(result_file_path)
        print(f"✅ [초압축 터널링 성공] {shard_idx}번: 통신량 {size_gz_mb:.2f}MB -> 실제 뼈대 복원 {size_raw_mb:.2f}MB 완료!")
        return True
    except Exception as e:
        print(f"❌ [에러 발생] {shard_idx}번 조각 다운로드 실패: {e}")
        return False

# 멀티스레딩으로 병렬 다운로드 (이더넷 + 와이파이 대역폭 극한 활용)
with ThreadPoolExecutor(max_workers=MAX_WORKERS) as executor:
    futures = {executor.submit(fetch_shard, i): i for i in range(1, TOTAL_SHARDS + 1)}
    
    for future in as_completed(futures):
        shard_idx = futures[future]
        try:
            future.result()
        except Exception as e:
            print(f"⚠️ {shard_idx}번 작업 중 예외 발생: {e}")

print("🎉 [병렬 작전 대성공] 모든 압축 데이터가 로컬에 훔쳐졌습니다!!")
