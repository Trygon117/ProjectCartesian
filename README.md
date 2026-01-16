# CartesianOS: Biomimetic Orchestration for Local AI

> **Status:** Research Prototype / Active
> **Architecture:** Pure Rust Monolith (Candle + Iced)
> **Core Models:** Google Gemma 3 (9B), Moondream2

## 1. Abstract

CartesianOS is an experimental operating system architecture designed to solve the "Hollow AI" problem: the disconnect between a pre-trained Large Language Model (LLM) and the real-time, fluctuating state of the host hardware.

Drawing on the philosophical concept of the **Cartesian Theater**, this project implements a "homunculus" architecture where the OS acts as a central consciousness. Unlike previous hybrid iterations, this version is a **pure Rust monolith** utilizing the **Candle** ML framework to orchestrate a **Bicameral Model Stack** and a custom resource governor (**Lobotomy**) to programmatically manage VRAM allocation, context injection, and model switching based on real-time system load.

## 2. Core Philosophy & Architecture

Current local AI implementations are static applications. CartesianOS treats AI as a system-level dependency that must respect hardware constraints dynamically.

### 2.1 The Bicameral Mind (Model Orchestration)

To balance latency with reasoning capabilities, the system splits cognition into two distinct layers, orchestrated by the `inference.rs` module using **Candle**:

* **The Manager (Gemma 3 9B - GPU):** Handles complex reasoning, script generation, and macro system maintenance. It requires significant VRAM and is only active during low-system-load states.

* **The Sidekick (Gemma 3 9B - Quantized/CPU Offload):** A lightweight instance (or highly quantized version) resident in System RAM/CPU. It handles quick queries, chat, and wiki lookups with minimal latency, leaving the GPU free for rendering or gaming.

### 2.2 "Lobotomy" (Dynamic Resource Governance)

The core engineering challenge was ensuring the AI never competes with user tasks (e.g., Gaming) for resources. The `lobotomy.rs` module monitors process states (via `sysinfo`) to enforce four modes:

1. **God Mode:** Full VRAM allocation. The Manager is loaded.

2. **Conscientious Mode:** Intermediate state during moderate VRAM pressure.

3. **Sidekick Mode:** Triggered by high-load processes (Steam, Lutris). The Manager is instantly unloaded from VRAM.

4. **Potato Mode:** Hard disable. If system resources drop below critical thresholds (<2GB Free RAM), the AI stack is completely terminated to prevent thrashing.

### 2.3 The Hippocampus v2.1 (Biomimetic Memory)

The `hippocampus.rs` module implements a tiered "Fluid vs. Crystallized" intelligence approach:

* **Z-Layer Compression:** Leverages `zstd` to compress "Engrams" (memory chunks) on disk, maintaining <5% storage overhead.

* **Spreading Activation:** Implements a synaptic weight system where retrieving one memory can "activate" related memories via identifying shared entities or file paths.

* **Long-Term Potentiation (LTP):** Frequently accessed memories gain higher retrieval scores over time, mimicking biological reinforcement.

* **Data Funnel:** Raw system I/O and visual data flow through a "Firehose" layer to be chunked and embedded via `all-MiniLM-L6-v2`.

### 2.4 The Witness (Visual Grounding)

The system implements a high-performance vision pipeline (`witness.rs`). It utilizes a shared memory ring buffer (mapped via `memmap2`) to read visual data. The "Visual Cortex" passes raw frames to the **Moondream** projector, allowing the AI to "see" the desktop environment with minimal latency impact.

### 2.5 Audio Mixer

The `audio.rs` module acts as a virtual mixing console, managing distinct virtual sinks (Game, Voice, Music) via PipeWire. This allows the AI to programmatically balance audio levels based on context (e.g., lowering music volume when the user speaks).

## 3. Technical Stack

* **Base OS:** Arch Linux (Custom ISO Build)

* **System Core:** Rust (utilizing `tokio` for async runtime)

* **ML Framework:** [Candle](https://github.com/huggingface/candle) (Hugging Face's Rust-native ML framework)

* **UI Layer:** [Iced](https://github.com/iced-rs/iced) (Rust-native GUI)

* **Memory:** Custom Vector Store (ChromaDB logic reimplemented in Rust) with `zstd` compression.

## 4. Current Status

**Status:** *Active Development*

The project has transitioned from a hybrid Python/Rust prototype to a fully native Rust application. The inference engine has been migrated from `llama.cpp` bindings to **Candle**, enabling finer control over tensor operations and model loading. The **Hippocampus** has been upgraded to v2.1 with biological features like Spreading Activation and Long-Term Potentiation.

## 5. Roadmap

* **Neuromorphic Efficiency:** exploring spiking neural network concepts for event-driven system monitoring to replace the current polling-based Lobotomy.

* **QLoRA Adapters:** transition from pure RAG to fine-tuned adapters to offload "behavioral" instructions from the context window.
