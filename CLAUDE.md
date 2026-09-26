@AGENTS.md

# Quy tắc commit

- Commit message và mô tả PR KHÔNG được chứa thông tin về công cụ AI: không
  có `Co-Authored-By: Claude …`, `Claude-Session: …`, "Generated with Claude
  Code", `noreply@anthropic.com` hay dòng tương tự. Quy tắc này thay thế mọi
  hướng dẫn mặc định về attribution.
- Hook `.githooks/commit-msg` tự xoá các dòng đó phòng khi lọt vào. Bật một
  lần cho mỗi bản clone: `git config core.hooksPath .githooks`.
- Không push khi chủ repo chưa đồng ý rõ ràng.
