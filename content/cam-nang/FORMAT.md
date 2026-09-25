# Chuẩn viết bài Cẩm nang

File này không nằm trong thư mục mục nào nên bước build bỏ qua nó.

Cẩm nang là thông tin THỰC TẾ cho người Việt ở Hàn Quốc: visa, trường đại
học, việc làm. Sai một con số (lệ phí, điều kiện, hạn nộp) là người đọc mất
tiền, mất thời gian, có khi mất cả visa. Nên chính xác đặt trên hết.

## Nguyên tắc nội dung

- **Chỉ viết điều có nguồn.** Mỗi con số, điều kiện, giấy tờ phải lấy từ
  nguồn chính thức hoặc nguồn đáng tin (HiKorea, 법무부 출입국·외국인정책본부,
  Study in Korea, trang của trường, EPS/고용노동부, Đại sứ quán). Không tìm
  được nguồn thì KHÔNG viết, đừng đoán.
- **Nói rõ thời điểm.** Luật và lệ phí đổi theo năm. Con số nào dễ đổi thì
  ghi kèm "(năm 2026)" hoặc "theo thông báo tháng …".
- **Không hứa hẹn.** Không viết "chắc chắn đậu", "dễ xin". Viết điều kiện và
  để người đọc tự đánh giá.
- **Không quảng cáo** trung tâm, công ty môi giới nào.
- Câu ngắn, đi thẳng vào việc. Không mở bài dài, không câu nối rỗng.
- Thuật ngữ tiếng Hàn giữ nguyên kèm nghĩa lần đầu: "외국인등록증 (thẻ
  người nước ngoài)". Người đọc sẽ phải điền đúng chữ Hàn trên giấy tờ.

## Cấu trúc thư mục

```
content/cam-nang/visa/<slug>.md       — mỗi loại visa một bài
content/cam-nang/truong/<slug>.md     — mỗi trường một bài
content/cam-nang/viec-lam/<slug>.md   — bài về việc làm
```

`slug`: chữ thường, không dấu, nối bằng gạch ngang (vd `d-2-du-hoc`).
Có thể đặt số đầu để sắp thứ tự (`01-d-4-hoc-tieng.md`), số bị bỏ khỏi slug.

## Phần khai báo đầu file

Mỗi dòng `khoá: giá trị`, giá trị là chữ trên MỘT dòng (không dùng danh sách
YAML nhiều dòng).

Chung cho mọi bài:

```
---
title: Visa D-2: du học đại học
summary: Một câu nói bài này giúp người đọc làm gì.
updated: 2026-09-25
---
```

Thêm cho bài **visa**:

```
code: D-2
```

Thêm cho bài **trường**:

```
nameKo: 고려대학교
city: Seoul
kind: Tư thục            (Tư thục | Quốc lập | Công lập)
certified: Có            (chứng nhận 교육국제화역량 năm gần nhất: Có | Không | Chưa rõ)
topik: 3                 (TOPIK tối thiểu vào hệ đại học; không yêu cầu thì 0. Có ngoại lệ theo ngành thì ghi bằng chữ, vd "Cấp 3 (ngành nghệ thuật: cấp 2)")
tuition: 4,5–6 triệu won/học kỳ
languageSchool: Có       (có trường tiếng 어학당 không)
website: https://…
```

## Thân bài

Dùng `##` cho các mục chính. Mục cuối cùng luôn là `## Nguồn`.

### Bài visa: các mục nên có

1. `## Dành cho ai`
2. `## Điều kiện chính`
3. `## Hồ sơ cần chuẩn bị`: danh sách dạng checklist, người đọc tick được
   trên app:
   ```
   - [ ] Hộ chiếu còn hạn
   - [ ] Đơn xin cấp visa (사증발급신청서)
   ```
4. `## Lệ phí và thời gian xử lý`
5. `## Các bước nộp hồ sơ`
6. `## Liên quan tới KIIP và TOPIK` (nếu có)
7. `## Lưu ý`
8. `## Nguồn`

### Bài trường: các mục nên có

`## Tổng quan`, `## Tuyển sinh sinh viên quốc tế`, `## Học phí và học bổng`,
`## Ký túc xá`, `## Trường tiếng (어학당)`, `## Nguồn`.

### Bài việc làm

Tự do theo chủ đề, dùng `##` chia mục, kết thúc bằng `## Nguồn`.

### Mục Nguồn

Danh sách link, ghi rõ tên trang:

```
## Nguồn

- [HiKorea — Hướng dẫn visa D-2](https://www.hikorea.go.kr/…)
- [Study in Korea — Học phí](https://www.studyinkorea.go.kr/…)
```
