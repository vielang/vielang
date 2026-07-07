import type { Metadata } from 'next';
import { StaticPageLayout } from '@/components/shared/StaticPageLayout';

export const metadata: Metadata = {
  title: 'Chính sách bảo mật · Privacy Policy | VieLang',
  description: 'VieLang thu thập, sử dụng và bảo vệ dữ liệu cá nhân của bạn như thế nào.',
};

const UPDATED = '2026-07-04';

export default function PrivacyPage() {
  return (
    <StaticPageLayout
      updated={UPDATED}
      title={{ vn: 'Chính sách bảo mật', en: 'Privacy Policy' }}
      intro={{
        vn: 'Chính sách này giải thích dữ liệu nào chúng tôi thu thập, sử dụng để làm gì, và bạn có quyền gì đối với dữ liệu của mình.',
        en: 'This policy explains what data we collect, how we use it, and the rights you have over your data.',
      }}
      sections={[
        {
          heading: { vn: '1. Dữ liệu chúng tôi thu thập', en: '1. What we collect' },
          bullets: [
            {
              vn: 'Thông tin tài khoản: tên, email, mật khẩu (đã băm), avatar, ngôn ngữ ưa thích.',
              en: 'Account info: name, email, hashed password, avatar, preferred language.',
            },
            {
              vn: 'Dữ liệu học tập: buổi học đã đặt, lịch rảnh, đánh giá, tài liệu, tin nhắn trong lớp.',
              en: 'Learning data: booked sessions, availability, reviews, materials, in-class messages.',
            },
            {
              vn: 'Dữ liệu kỹ thuật: địa chỉ IP, loại trình duyệt, cookie phiên (dùng cho xác thực).',
              en: 'Technical data: IP address, browser type, session cookies (used for authentication).',
            },
            {
              vn: 'Nhật ký video/audio: LiveKit chỉ streaming — chúng tôi KHÔNG ghi âm hay ghi hình buổi học trừ khi bạn được thông báo và đồng ý.',
              en: 'Video/audio streams: LiveKit is streaming-only — we do NOT record sessions unless you are notified and consent.',
            },
          ],
        },
        {
          heading: { vn: '2. Chúng tôi dùng dữ liệu để làm gì', en: '2. How we use your data' },
          bullets: [
            {
              vn: 'Cung cấp và cải thiện dịch vụ (đặt lịch, phòng học, nhắc nhở qua email).',
              en: 'Provide and improve the service (bookings, classrooms, email reminders).',
            },
            {
              vn: 'Ghép học viên với giáo viên phù hợp dựa trên trình độ và mục tiêu.',
              en: 'Match students with tutors based on level and goals.',
            },
            {
              vn: 'Ngăn chặn gian lận, lạm dụng, và duy trì an ninh nền tảng.',
              en: 'Prevent fraud and abuse, and maintain platform security.',
            },
            {
              vn: 'Tuân thủ nghĩa vụ pháp lý khi cơ quan có thẩm quyền yêu cầu hợp pháp.',
              en: 'Comply with legal obligations when a lawful request is made by authorities.',
            },
          ],
        },
        {
          heading: { vn: '3. Chia sẻ với bên thứ ba', en: '3. Sharing with third parties' },
          paragraphs: [
            {
              vn: 'Chúng tôi KHÔNG bán dữ liệu cá nhân. Chúng tôi có sử dụng nhà cung cấp dịch vụ (Supabase cho cơ sở dữ liệu, LiveKit cho video, Resend cho email, Sentry cho báo lỗi). Các nhà cung cấp này chỉ được xử lý dữ liệu theo hướng dẫn của chúng tôi.',
              en: 'We do NOT sell personal data. We use service providers (Supabase for the database, LiveKit for video, Resend for email, Sentry for error reporting). These providers process data only under our instructions.',
            },
          ],
        },
        {
          heading: { vn: '4. Cookie', en: '4. Cookies' },
          paragraphs: [
            {
              vn: 'Chúng tôi dùng cookie phiên (HTTP-only) để giữ trạng thái đăng nhập. Không có cookie quảng cáo hay tracking bên thứ ba.',
              en: 'We use HTTP-only session cookies to keep you signed in. No advertising or third-party tracking cookies.',
            },
          ],
        },
        {
          heading: { vn: '5. Bảo mật', en: '5. Security' },
          paragraphs: [
            {
              vn: 'Mọi giao tiếp giữa trình duyệt và máy chủ đều qua HTTPS. Mật khẩu được băm bằng thuật toán tiêu chuẩn ngành. Chúng tôi giám sát lỗi và bất thường thông qua công cụ observability để phản ứng nhanh với sự cố.',
              en: 'All browser-server communication runs over HTTPS. Passwords are hashed with an industry-standard algorithm. We monitor errors and anomalies via observability tooling so we can respond to incidents quickly.',
            },
          ],
        },
        {
          heading: { vn: '6. Quyền của bạn', en: '6. Your rights' },
          bullets: [
            {
              vn: 'Truy cập, chỉnh sửa hoặc xoá thông tin cá nhân qua trang Cá nhân.',
              en: 'Access, edit or delete your personal info from your profile page.',
            },
            {
              vn: 'Yêu cầu xoá tài khoản vĩnh viễn — chúng tôi sẽ xử lý trong vòng 30 ngày.',
              en: 'Request permanent account deletion — we will process within 30 days.',
            },
            {
              vn: 'Nhận bản sao dữ liệu bạn đã cung cấp cho VieLang.',
              en: 'Receive a copy of the data you have provided to VieLang.',
            },
          ],
        },
        {
          heading: { vn: '7. Lưu trữ dữ liệu', en: '7. Data retention' },
          paragraphs: [
            {
              vn: 'Dữ liệu tài khoản được giữ cho đến khi bạn yêu cầu xoá. Nhật ký email và hoạt động được giữ tối đa 12 tháng phục vụ mục đích hỗ trợ và phòng chống gian lận.',
              en: 'Account data is kept until you request deletion. Email and activity logs are retained for up to 12 months for support and fraud-prevention purposes.',
            },
          ],
        },
        {
          heading: { vn: '8. Trẻ em', en: '8. Children' },
          paragraphs: [
            {
              vn: 'Dịch vụ không dành cho trẻ dưới 13 tuổi. Nếu phát hiện chúng tôi vô tình thu thập dữ liệu từ trẻ dưới 13, chúng tôi sẽ xoá.',
              en: 'The service is not directed to children under 13. If we discover we have inadvertently collected data from a child under 13, we will delete it.',
            },
          ],
        },
        {
          heading: { vn: '9. Thay đổi', en: '9. Changes' },
          paragraphs: [
            {
              vn: 'Chúng tôi có thể cập nhật chính sách. Ngày cập nhật ở đầu trang phản ánh phiên bản mới nhất.',
              en: 'We may update this policy. The revision date at the top reflects the latest version.',
            },
          ],
        },
        {
          heading: { vn: '10. Liên hệ', en: '10. Contact' },
          paragraphs: [
            {
              vn: 'Câu hỏi về dữ liệu cá nhân, vui lòng liên hệ qua trang Liên hệ.',
              en: 'Questions about your personal data? Reach out via the Contact page.',
            },
          ],
        },
      ]}
    />
  );
}
