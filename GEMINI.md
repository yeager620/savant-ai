
# Savant AI Project Summary

This document provides a high-level overview of the Savant AI project, including its goals, architecture, and key components.

## Project Goal

Savant AI aims to be an "invisible" AI assistant with multimodal intelligence. It captures and analyzes both audio and video from the user's environment in real-time, providing a comprehensive understanding of the user's activities. The ultimate goal is to provide proactive assistance and a "smart memory" that can be queried using natural language.

## Core Functionality

*   **Real-time Audio Transcription:** Captures system audio and microphone input, transcribes it using Whisper, and identifies different speakers.
*   **Intelligent Screen Capture:** Takes screenshots at regular intervals, performs OCR to extract text, and uses computer vision to identify applications and user activities.
*   **Multimodal Correlation:** Synchronizes audio and video data to create a unified timeline of events.
*   **Smart Database:** Stores all captured data in a local SQLite database, optimized for efficient querying.
*   **Natural Language Queries:** Allows users to query the database using plain English through a chat interface, powered by LLMs.
*   **UNIX-like CLI Tools:** The system is built as a collection of small, composable command-line tools that follow the UNIX philosophy.

## Architecture

The project follows a modular, data-pipeline architecture:

1.  **Daemons:** Separate background processes for audio and video capture (`savant-audio-daemon` and `savant-video-daemon`).
2.  **Processing Tools:** A suite of CLI tools for specific tasks:
    *   `savant-ocr`: Text extraction from images.
    *   `savant-vision`: Computer vision analysis.
    *   `savant-transcribe`: Audio transcription.
    *   `savant-sync`: Multimodal data synchronization.
    *   `savant-llm`: LLM inference.
    *   `savant-db`: Database management.
3.  **Data Storage:** A central SQLite database (`transcripts.db`) stores all processed data.
4.  **MCP (Model Context Protocol) Server:** A JSON-RPC server (`savant-mcp`) that exposes the database to LLMs, allowing for natural language queries.
5.  **Frontend:** A Tauri-based desktop application with a Leptos WASM frontend provides a GUI for interacting with the system.

## Design Philosophy

*   **UNIX Philosophy:** The system is designed as a collection of small, independent tools that "do one thing well" and can be composed together using pipes. This promotes modularity, testability, and flexibility.
*   **Privacy First:** All data is processed and stored locally by default. The user has explicit control over what data is shared with external services.
*   **Composability:** The CLI tools are designed to be easily combined to create new workflows.
*   **Text Streams:** JSON is used as the standard data interchange format between tools.

## Key Technologies

*   **Rust:** The entire backend is written in Rust for performance and safety.
*   **Tauri:** Used for the desktop application framework.
*   **Leptos:** A Rust-based web framework for the WASM frontend.
*   **Whisper:** Used for speech-to-text transcription.
*   **Tesseract:** Used for OCR.
*   **Ollama:** Used for running local LLMs.
*   **SQLite:** Used for the local database.

## Project Status & Future Work (from TODO.md)

The project is currently in a functional state, with many of the core features implemented. The `TODO.md` file outlines the following areas for future development:

*   **Smart Database:** Enhance the screen capture processing pipeline to create a more structured and meaningful representation of the on-screen data.
*   **Improved GUI:** Add more features to the Tauri application, including a dropdown menu, daemon control, and analytics.
*   **Unified Chatbot:** Create a single chat interface for querying both audio and visual data.
*   **Containerization and Release:** Develop a plan for packaging and distributing the application.
*   **Cross-platform Support:** Extend the project to work on Windows and Linux.
*   **Unit Tests:** Add more comprehensive unit tests for the data pipelines.
*   **LLM Integration:** Add support for more LLM models and providers.
