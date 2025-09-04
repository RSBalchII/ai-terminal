#!/usr/bin/env python3
"""
Automated test script for AI Terminal chat functionality
Tests the Ollama integration directly
"""

import json
import time
import requests
from typing import Optional, Dict, Any

class ChatTester:
    def __init__(self, ollama_url: str = "http://localhost:11434"):
        self.ollama_url = ollama_url
        self.test_results = []
        
    def log_result(self, test_name: str, success: bool, details: str = ""):
        """Log a test result"""
        result = {
            "test": test_name,
            "success": success,
            "details": details,
            "timestamp": time.time()
        }
        self.test_results.append(result)
        
        # Print result
        status = "✅ PASS" if success else "❌ FAIL"
        print(f"{status}: {test_name}")
        if details:
            print(f"  Details: {details}")
    
    def test_ollama_connection(self) -> bool:
        """Test if Ollama is accessible"""
        try:
            response = requests.get(f"{self.ollama_url}/api/tags", timeout=5)
            if response.status_code == 200:
                models = response.json().get("models", [])
                self.log_result(
                    "Ollama Connection",
                    True,
                    f"Found {len(models)} models: {', '.join([m['name'] for m in models])}"
                )
                return True
            else:
                self.log_result("Ollama Connection", False, f"Status code: {response.status_code}")
                return False
        except Exception as e:
            self.log_result("Ollama Connection", False, str(e))
            return False
    
    def test_model_availability(self, model_name: str = "nemotron-mini:4b-instruct-q8_0") -> bool:
        """Test if a specific model is available"""
        try:
            response = requests.get(f"{self.ollama_url}/api/tags", timeout=5)
            if response.status_code == 200:
                models = response.json().get("models", [])
                model_names = [m['name'] for m in models]
                if model_name in model_names:
                    self.log_result("Model Availability", True, f"Model '{model_name}' is available")
                    return True
                else:
                    self.log_result(
                        "Model Availability",
                        False,
                        f"Model '{model_name}' not found. Available: {', '.join(model_names)}"
                    )
                    return False
        except Exception as e:
            self.log_result("Model Availability", False, str(e))
            return False
    
    def test_simple_generation(self, model_name: str = "nemotron-mini:4b-instruct-q8_0") -> bool:
        """Test a simple text generation"""
        try:
            payload = {
                "model": model_name,
                "prompt": "Say 'Hello, AI Terminal!' and nothing else.",
                "stream": False,
                "options": {
                    "temperature": 0.1,
                    "num_predict": 50
                }
            }
            
            start_time = time.time()
            response = requests.post(
                f"{self.ollama_url}/api/generate",
                json=payload,
                timeout=45
            )
            duration = time.time() - start_time
            
            if response.status_code == 200:
                result = response.json()
                generated_text = result.get("response", "")
                self.log_result(
                    "Simple Generation",
                    True,
                    f"Generated in {duration:.2f}s: '{generated_text[:50]}...'"
                )
                return True
            else:
                self.log_result("Simple Generation", False, f"Status code: {response.status_code}")
                return False
        except Exception as e:
            self.log_result("Simple Generation", False, str(e))
            return False
    
    def test_streaming_generation(self, model_name: str = "nemotron-mini:4b-instruct-q8_0") -> bool:
        """Test streaming text generation"""
        try:
            payload = {
                "model": model_name,
                "prompt": "Count from 1 to 5.",
                "stream": True,
                "options": {
                    "temperature": 0.1,
                    "num_predict": 50
                }
            }
            
            tokens = []
            start_time = time.time()
            
            response = requests.post(
                f"{self.ollama_url}/api/generate",
                json=payload,
                stream=True,
                timeout=45
            )
            
            if response.status_code == 200:
                for line in response.iter_lines():
                    if line:
                        try:
                            chunk = json.loads(line)
                            if "response" in chunk:
                                tokens.append(chunk["response"])
                        except json.JSONDecodeError:
                            pass
                
                duration = time.time() - start_time
                full_response = "".join(tokens)
                self.log_result(
                    "Streaming Generation",
                    True,
                    f"Received {len(tokens)} tokens in {duration:.2f}s"
                )
                return True
            else:
                self.log_result("Streaming Generation", False, f"Status code: {response.status_code}")
                return False
        except Exception as e:
            self.log_result("Streaming Generation", False, str(e))
            return False
    
    def test_conversation_context(self, model_name: str = "nemotron-mini:4b-instruct-q8_0") -> bool:
        """Test conversation with context"""
        try:
            # First message
            payload1 = {
                "model": model_name,
                "prompt": "My name is TestBot. What is my name?",
                "stream": False,
                "options": {"temperature": 0.1, "num_predict": 50}
            }
            
            response1 = requests.post(
                f"{self.ollama_url}/api/generate",
                json=payload1,
                timeout=45
            )
            
            if response1.status_code != 200:
                self.log_result("Conversation Context", False, "First message failed")
                return False
            
            result1 = response1.json()
            context = result1.get("context", [])
            
            # Second message with context
            payload2 = {
                "model": model_name,
                "prompt": "What did I just tell you my name was?",
                "context": context,
                "stream": False,
                "options": {"temperature": 0.1, "num_predict": 50}
            }
            
            response2 = requests.post(
                f"{self.ollama_url}/api/generate",
                json=payload2,
                timeout=45
            )
            
            if response2.status_code == 200:
                result2 = response2.json()
                response_text = result2.get("response", "").lower()
                
                # Check if the model remembers the name
                if "testbot" in response_text:
                    self.log_result(
                        "Conversation Context",
                        True,
                        "Model correctly maintained context"
                    )
                    return True
                else:
                    self.log_result(
                        "Conversation Context",
                        False,
                        f"Model did not remember context: '{response_text[:50]}...'"
                    )
                    return False
            else:
                self.log_result("Conversation Context", False, f"Second message failed")
                return False
        except Exception as e:
            self.log_result("Conversation Context", False, str(e))
            return False
    
    def run_all_tests(self) -> None:
        """Run all chat tests"""
        print("\n" + "="*60)
        print("🚀 AI Terminal Chat Testing Suite")
        print("="*60 + "\n")
        
        # Test connection first
        if not self.test_ollama_connection():
            print("\n⚠️ Cannot proceed without Ollama connection")
            return
        
        # Get available model
        response = requests.get(f"{self.ollama_url}/api/tags", timeout=5)
        models = response.json().get("models", [])
        if not models:
            print("\n⚠️ No models available in Ollama")
            return
        
        model_name = models[0]["name"]
        print(f"\n📦 Using model: {model_name}\n")
        
        # Run tests
        tests = [
            ("Model Availability", lambda: self.test_model_availability(model_name)),
            ("Simple Generation", lambda: self.test_simple_generation(model_name)),
            ("Streaming Generation", lambda: self.test_streaming_generation(model_name)),
            ("Conversation Context", lambda: self.test_conversation_context(model_name))
        ]
        
        for test_name, test_func in tests:
            print(f"\nTesting: {test_name}")
            print("-" * 40)
            test_func()
            time.sleep(1)  # Small delay between tests
        
        # Summary
        print("\n" + "="*60)
        print("📊 Test Summary")
        print("="*60)
        
        passed = sum(1 for r in self.test_results if r["success"])
        total = len(self.test_results)
        
        print(f"\nResults: {passed}/{total} tests passed")
        
        if passed == total:
            print("✅ All tests passed! Chat functionality is working correctly.")
        else:
            print("❌ Some tests failed. Please check the details above.")
            failed_tests = [r["test"] for r in self.test_results if not r["success"]]
            print(f"Failed tests: {', '.join(failed_tests)}")

if __name__ == "__main__":
    tester = ChatTester()
    tester.run_all_tests()
