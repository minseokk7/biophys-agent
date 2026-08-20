import os
import time
import sys

try:
    import torch
    from transformers import AutoModelForCausalLM, AutoTokenizer
except ImportError:
    print("❌ [오류] transformers 또는 torch 라이브러리가 설치되어 있지 않습니다.")
    print("터미널에서 다음 명령어를 실행해 주세요: pip install transformers torch accelerate bitsandbytes")
    sys.exit(1)

def test_fuse_real_world():
    print("==================================================")
    print("🔍 [실제 환경 테스트] Akahsizrr/fuse-1-Lite-4bit 가동")
    print("==================================================")
    
    # 4-bit 경량화 버전 사용 (약 3.36GB)
    model_id = "Akahsizrr/fuse-1-Lite-4bit"
    
    try:
        print(f"[1/3] Tokenizer 다운로드 및 로딩 중: {model_id} ...")
        tokenizer = AutoTokenizer.from_pretrained(model_id, trust_remote_code=True)
        
        print(f"[2/3] Model 가중치(3.36GB) 다운로드 및 로딩 중...")
        print("  >> (인터넷 속도에 따라 수 분이 소요될 수 있습니다. 터미널 출력을 기다려 주세요.)")
        
        # Windows 로컬 환경을 고려하여 GPU가 없으면 CPU로 Fallback
        device = "cuda" if torch.cuda.is_available() else "cpu"
        print(f"  >> 감지된 디바이스: {device.upper()}")
        
        model = AutoModelForCausalLM.from_pretrained(
            model_id,
            device_map="auto" if device == "cuda" else None,
            trust_remote_code=True,
            torch_dtype=torch.float16 if device == "cuda" else torch.float32
        )
        
        print("\n[3/3] 🟢 로딩 완료! 실제 추론(Inference) 벤치마크 진행")
        
        # 코딩 전문가(MoE) 회로를 강제로 깨우는 프롬프트
        prompt = "Write a highly optimized Python function to check if a number is prime, and explain how it works."
        print(f"\n[유저 프롬프트] {prompt}\n")
        
        # 모델 포맷에 맞춘 챗 템플릿 적용 (필요시)
        inputs = tokenizer(prompt, return_tensors="pt").to(model.device)
        
        print("⏳ 코딩 전문가 모듈 활성화 및 추론 중...")
        start_time = time.time()
        
        # 실제 생성 (최대 256토큰)
        outputs = model.generate(**inputs, max_new_tokens=256, temperature=0.2)
        end_time = time.time()
        
        # 결과 디코딩
        response = tokenizer.decode(outputs[0][inputs["input_ids"].shape[-1]:], skip_special_tokens=True)
        
        print("\n==================================================")
        print("✅ [생성된 코드 및 답변]")
        print("--------------------------------------------------")
        print(response)
        print("==================================================")
        
        # 실측 성능 분석
        gen_time = end_time - start_time
        tokens_generated = len(outputs[0]) - inputs["input_ids"].shape[-1]
        tps = tokens_generated / gen_time if gen_time > 0 else 0
        
        print("\n📊 [실측 성능 분석]")
        print(f" - 소요 시간: {gen_time:.2f}초")
        print(f" - 생성 토큰: {tokens_generated} 토큰")
        print(f" - 실제 속도: {tps:.2f} Tokens/sec")
        if device == "cpu":
            print(" ⚠️ (경고: 현재 CPU로 연산되어 속도가 느릴 수 있습니다. CUDA 환경 시 대폭 상승합니다.)")
        print("==================================================")
        
    except Exception as e:
        print(f"\n❌ [오류 발생] 실제 테스트 중 문제가 발생했습니다: {e}")
        print(" >> 허깅페이스 접속 차단, VRAM 부족, 또는 Windows 환경의 bitsandbytes 호환성 문제일 수 있습니다.")

if __name__ == "__main__":
    test_fuse_real_world()
