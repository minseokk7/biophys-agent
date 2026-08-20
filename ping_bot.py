import time
import requests

# 유저님의 허깅페이스 스페이스 URL을 여기에 입력하세요.
SPACE_URL = "https://minseokk7-biophys-agent.hf.space"

print(f"🤖 [Ping Bot] 가동 시작! {SPACE_URL} 서버를 1분마다 찔러서 재우지 않습니다.")

ping_count = 0
while True:
    try:
        response = requests.get(SPACE_URL, timeout=10)
        ping_count += 1
        print(f"[{ping_count}회 찌르기] 상태: {response.status_code} - ☁️ 서버가 잠들지 않고 열일 중입니다!")
    except Exception as e:
        print(f"❌ 접속 오류 발생: {e}")
    
    # 60초 대기
    time.sleep(60)
