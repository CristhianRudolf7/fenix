# Dictation and text conversation

The chat microphone uses browser speech recognition to transcribe into the draft. Review the text before sending; dictation never submits automatically. Browser support and microphone permissions determine availability. The application has no Live call mode or speech synthesis.

Web chat and Telegram retain a fast conversational model independent of the master. Its configuration uses `/api/model-roles/fast`. Tasks, plans and subagents run independently of dictation. Existing installations retain historical fast-model settings and usage during migration.

Mocked speech and provider tests validate code behavior, not real acoustic quality.
