# ORION GPU Tower — Setup Guide

## Tower Specs

| Component | Spec |
|-----------|------|
| **CPU** | AMD Threadripper 7970X (32 cores) |
| **GPU** | 2× NVIDIA RTX 4090 (48GB VRAM total) |
| **RAM** | 128GB DDR5 ECC |
| **Storage** | High-speed NVMe SSD |

This is the machine that will run ORION's fully personal brain — no third-party dependencies.

---

## Phase 1: OS & Environment

### Step 1: Install OS

```
Option A: Ubuntu Server 24.04 LTS (Recommended)
  - Best GPU support
  - Lower overhead than Windows
  - Better for running inference 24/7

Option B: Ubuntu Desktop 24.04 LTS
  - GUI if you prefer visual interface
  - Same GPU support as Server
```

### Step 2: Initial Setup Commands

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install essential tools
sudo apt install -y git curl wget htop neofetch net-tools
```

### Step 3: Install CUDA + GPU Drivers

```bash
# Option A: NVIDIA CUDA Toolkit (recommended)
wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2404/x86_64/cuda-keyring_1.1-1_all.deb
sudo dpkg -i cuda-keyring_1.1-1_all.deb
sudo apt update
sudo apt install -y cuda-toolkit-12-8

# Add to PATH
echo 'export PATH=/usr/local/cuda/bin:$PATH' >> ~/.bashrc
echo 'export LD_LIBRARY_PATH=/usr/local/cuda/lib64:$LD_LIBRARY_PATH' >> ~/.bashrc
source ~/.bashrc

# Verify
nvidia-smi
```

Expected output:
```
+-----------------------------------------------------------------------------+
| NVIDIA-SMI ...  Driver Version: ...  CUDA Version: 12.8                     |
+-------------------------------+----------------------+----------------------+
| GPU  Name        Persistence-M| Bus-Id        Disp.A | Volatile Uncorr. ECC |
|===============================+======================+======================|
|   0  NVIDIA RTX 4090         ...                      |   24GB VRAM          |
|   1  NVIDIA RTX 4090         ...                      |   24GB VRAM          |
+-------------------------------+----------------------+----------------------+
```

---

## Phase 2: Python Environment

### Step 4: Install Python

```bash
# Install Python 3.11
sudo apt install -y python3.11 python3.11-venv python3-pip

# Create virtual environment
python3.11 -m venv ~/orion-env
source ~/orion-env/bin/activate

# Make sure it activates on login
echo 'source ~/orion-env/bin/activate' >> ~/.bashrc
```

### Step 5: Install PyTorch + Dependencies

```bash
# Activate environment
source ~/orion-env/bin/activate

# Install PyTorch with CUDA 12.1 support
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121

# Verify GPU access
python -c "import torch; print(f'CUDA available: {torch.cuda.is_available()}'); print(f'GPU count: {torch.cuda.device_count()}')"
```

Expected output:
```
CUDA available: True
GPU count: 2
```

### Step 6: Install Inference Dependencies

```bash
# Install the full stack
pip install \
  fastapi \
  uvicorn[standard] \
  transformers \
  accelerate \
  sentencepiece \
  protobuf \
  huggingface-hub \
  bitsandbytes \
  scipy \
  safetensors
```

---

## Phase 3: ORION Inference Server

### Step 7: Create Project Structure

```bash
mkdir -p ~/orion-inference
cd ~/orion-inference
mkdir models routes
```

### Step 8: Download Models

```bash
# Login to HuggingFace (optional, for gated models)
huggingface-cli login

# Create model directories
mkdir -p models/deepseek-r1-70b
mkdir -p models/codeqwen-14b
```

**Model downloads happen via Python at server startup or manually.**

### Step 9: Start the Inference Server

```bash
# Start server
cd ~/orion-inference
source ~/orion-env/bin/activate
python server.py

# Server runs on:
# http://localhost:11434/api/chat   (Ollama-compatible endpoint)
# http://localhost:11434/api/generate
```

### Step 10: Test the Server

```bash
# Test basic chat
curl -X POST http://localhost:11434/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "model": "deepseek-r1-70b",
    "messages": [{"role": "user", "content": "Hello, who are you?"}]
  }'

# Test streaming
curl -X POST http://localhost:11434/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "model": "deepseek-r1-70b",
    "messages": [{"role": "user", "content": "Write a simple Python function"}],
    "stream": true
  }'
```

---

## Phase 4: Connect ORION Desktop App to Tower

### Step 11: Configure ORION to Use Tower Instead of Ollama

Edit `CONFIG/settings.json`:
- Set `ollama.host` to tower's IP: `http://192.168.x.x:11434`
- Set `ollama.enabled: true`
- Set `routing.localFirst: true`

### Step 12: Remote Access (Cloudflare Tunnel)

```bash
# Install cloudflared on tower
curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64.deb -o cloudflared.deb
sudo dpkg -i cloudflared.deb

# Authenticate
cloudflared tunnel login

# Create tunnel
cloudflared tunnel create orion

# Configure tunnel (edit ~/.cloudflared/config.yml)
```

---

## Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────────┐
│                     GPU TOWER (India)                                │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────┐      │
│  │                  Python Inference Server                    │      │
│  │  Port 11434 (Ollama-compatible API)                      │      │
│  │                                                          │      │
│  │  ┌──────────────────┐    ┌──────────────────┐          │      │
│  │  │ GPU 0 (RTX 4090)  │    │ GPU 1 (RTX 4090)  │          │      │
│  │  │ DeepSeek R1 70B   │    │ CodeQwen 14B      │          │      │
│  │  │ (Chat/Reasoning) │    │ (Code Generation) │          │      │
│  │  └──────────────────┘    └──────────────────┘          │      │
│  │                                                          │      │
│  │  Future: Image Gen (Stable Diffusion on GPU 1)          │      │
│  └──────────────────────────────────────────────────────────┘      │
│                              │                                       │
│                              ▼                                       │
│                    ┌──────────────────┐                              │
│                    │ Cloudflare Tunnel │                              │
│                    └──────────────────┘                              │
│                              │                                       │
└──────────────────────────────┼───────────────────────────────────────┘
                               │
                               ▼
┌──────────────────────────────────────────────────────────────────────┐
│              ACCESS FROM ANYWHERE (Browser + Password)              │
│                                                                      │
│  • https://orion-{name}.cloudflare.app                              │
│  • Phone / Laptop / Any device with browser                         │
│  • Login with password                                              │
└──────────────────────────────────────────────────────────────────────┘
```

---

## Model Strategy for 2× RTX 4090

| GPU | Model | VRAM Use | Purpose |
|-----|-------|----------|---------|
| **GPU 0** | DeepSeek R1 70B | ~40GB | Primary chat, reasoning |
| **GPU 1** | CodeQwen 14B | ~28GB | Code generation |
| **GPU 1** (switch) | Stable Diffusion XL | ~8GB | Image generation |
| **Future** | Video model | ~20GB | Video generation |

You can hot-swap models on GPU 1 based on the task.

---

## Summary: What Changes After Tower

| Before (Current Laptop) | After (GPU Tower) |
|-------------------------|-------------------|
| Small models (1.5b-8b) | Large models (14b-70b) |
| Free APIs (Groq/Gemini) | Custom PyTorch inference |
| Paid APIs planned | No third-party APIs |
| No local image gen | Local Stable Diffusion |
| Ollama as middle layer | Direct PyTorch control |
