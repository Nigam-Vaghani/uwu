import os
import sys

# 1. Load libraries and handle missing dependencies
try:
    from groq import Groq
    from dotenv import load_dotenv
except ImportError:
    print("Error: Missing required libraries.")
    print("Please run: pip install groq python-dotenv")
    sys.exit(1)

# 2. Explicitly load the .env file from the current directory
load_dotenv()

def list_groq_models():
    # Retrieve the API key from your environment variables
    api_key = os.environ.get("GROQ_API_KEY")
    
    # Check if the key is missing or still set to the placeholder string
    if not api_key or api_key == "gsk_your_api_key_here":
        print("Error: GROQ_API_KEY not detected in your environment or .env file.")
        print("Please ensure your .env file is in this exact folder and contains:")
        print("GROQ_API_KEY=gsk_your_actual_key")
        sys.exit(1)

    try:
        # Initialize client
        client = Groq(api_key=api_key)
        
        # Request active models
        models_list = client.models.list()
        
        print("\n=== SUCCESS: Connected to Groq ===")
        print(f"{'MODEL ID':<35} | {'OWNED BY':<12} | {'ACTIVE'}")
        print("-" * 60)
        
        for model in models_list.data:
            print(f"{model.id:<35} | {model.owned_by:<12} | {model.active}")
            
    except Exception as e:
        print(f"\nAPI Error: Could not fetch models. Verify your API key.")
        print(f"Details: {e}")

if __name__ == "__main__":
    list_groq_models()
