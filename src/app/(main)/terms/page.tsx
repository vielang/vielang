import type { Metadata } from 'next';
import { StaticPageLayout } from '@/components/shared/StaticPageLayout';

export const metadata: Metadata = {
  title: 'Điều khoản · Terms of Service | VieLang',
  description: 'Điều khoản sử dụng dịch vụ VieLang.',
};

const UPDATED = '2026-07-04';

export default function TermsPage() {
  return (
    <StaticPageLayout
      updated={UPDATED}
      title={{
        vn: 'Điều khoản sử dụng',
        en: 'Terms of Service',
      }}
      intro={{
        vn: 'Khi truy cập hoặc sử dụng VieLang, bạn đồng ý với các điều khoản dưới đây. Vui lòng đọc kỹ trước khi đăng ký tài khoản hoặc đặt buổi học.',
        en: 'By accessing or using VieLang, you agree to the terms below. Please read them carefully before creating an account or booking a session.',
      }}
      sections={[
        {
          heading: { vn: '1. Về dịch vụ', en: '1. About the service' },
          paragraphs: [
            {
              vn: 'VieLang là nền tảng học tiếng Anh 1-on-1 và nhóm nhỏ qua video call với đội ngũ giáo viên đã được kiểm duyệt. Chúng tôi cung cấp công cụ đặt lịch, phòng học trực tuyến, và tài liệu học đi kèm khoá học.',
              en: 'VieLang is a live-video English-tutoring platform offering both 1-on-1 sessions and small group rooms with vetted tutors. We provide booking tools, an online classroom, and course materials that accompany each session.',
            },
          ],
        },
        {
          heading: { vn: '2. Tài khoản', en: '2. Your account' },
          bullets: [
            {
              vn: 'Bạn phải cung cấp thông tin chính xác khi đăng ký và cập nhật khi có thay đổi.',
              en: 'You must provide accurate information when registering and keep it up to date.',
            },
            {
              vn: 'Bạn chịu trách nhiệm bảo mật mật khẩu và mọi hoạt động phát sinh từ tài khoản của mình.',
              en: 'You are responsible for keeping your password confidential and for any activity that occurs on your account.',
            },
            {
              vn: 'Người dùng phải từ 13 tuổi trở lên. Người dưới 18 cần có sự đồng ý của phụ huynh hoặc người giám hộ.',
              en: 'Users must be at least 13 years old. Users under 18 need consent from a parent or guardian.',
            },
          ],
        },
        {
          heading: { vn: '3. Đặt lịch & thanh toán', en: '3. Booking & payment' },
          paragraphs: [
            {
              vn: 'Khi bạn đặt một buổi học 1-on-1, VieLang giữ chỗ với giáo viên trong khung giờ đó. Thanh toán được xử lý qua các cổng thanh toán mà chúng tôi tích hợp; các buổi group free-talk hiện miễn phí.',
              en: 'Booking a 1-on-1 session reserves the tutor for that time slot. Payments run through our integrated payment providers; group free-talk rooms are currently free to join.',
            },
          ],
        },
        {
          heading: { vn: '4. Huỷ & hoàn tiền', en: '4. Cancellation & refunds' },
          paragraphs: [
            {
              vn: 'Xem chi tiết trong Chính sách hoàn tiền. Tóm tắt: bạn có thể huỷ hoặc đổi lịch buổi học ít nhất 24 giờ trước giờ bắt đầu để được hoàn tiền đầy đủ.',
              en: 'See the Refund Policy for details. Summary: you can cancel or reschedule a session up to 24 hours before start time for a full refund.',
            },
          ],
        },
        {
          heading: { vn: '5. Ứng xử', en: '5. Acceptable use' },
          bullets: [
            {
              vn: 'Không đăng nội dung vi phạm pháp luật, kích động thù ghét, quấy rối, hoặc mang tính khiêu dâm.',
              en: 'Do not post content that is illegal, hateful, harassing, or sexually explicit.',
            },
            {
              vn: 'Không cố gắng truy cập trái phép tài khoản, phòng học, hoặc dữ liệu của người khác.',
              en: 'Do not attempt to access accounts, rooms, or data that are not yours.',
            },
            {
              vn: 'VieLang có quyền tạm ngưng hoặc chấm dứt tài khoản vi phạm mà không cần thông báo trước.',
              en: 'VieLang may suspend or terminate accounts that violate these terms without prior notice.',
            },
          ],
        },
        {
          heading: { vn: '6. Sở hữu trí tuệ', en: '6. Intellectual property' },
          paragraphs: [
            {
              vn: 'Tài liệu học do giáo viên tải lên vẫn thuộc quyền sở hữu của giáo viên đó. Bạn chỉ được sử dụng cho mục đích học tập cá nhân, không được sao chép hoặc phân phối lại.',
              en: 'Course materials uploaded by tutors remain the property of that tutor. You may use them for your personal study only — do not reproduce or redistribute.',
            },
          ],
        },
        {
          heading: { vn: '7. Giới hạn trách nhiệm', en: '7. Limitation of liability' },
          paragraphs: [
            {
              vn: 'VieLang cung cấp dịch vụ trên cơ sở "hiện trạng". Trong phạm vi tối đa cho phép của pháp luật, chúng tôi không chịu trách nhiệm về thiệt hại gián tiếp, ngẫu nhiên, hoặc mất lợi nhuận phát sinh từ việc sử dụng dịch vụ.',
              en: 'VieLang provides the service on an "as-is" basis. To the maximum extent permitted by law, we are not liable for indirect, incidental, or consequential damages arising from your use of the service.',
            },
          ],
        },
        {
          heading: { vn: '8. Thay đổi điều khoản', en: '8. Changes to these terms' },
          paragraphs: [
            {
              vn: 'Chúng tôi có thể cập nhật điều khoản theo thời gian. Bản mới sẽ được công bố tại trang này với ngày cập nhật ở phía trên; việc tiếp tục sử dụng dịch vụ đồng nghĩa với việc bạn chấp thuận bản mới.',
              en: 'We may update these terms from time to time. Updated versions are posted on this page with the revision date shown above; continued use of the service constitutes acceptance of the new terms.',
            },
          ],
        },
        {
          heading: { vn: '9. Liên hệ', en: '9. Contact' },
          paragraphs: [
            {
              vn: 'Câu hỏi về điều khoản, vui lòng liên hệ qua trang Liên hệ.',
              en: 'Questions about these terms? Reach out via the Contact page.',
            },
          ],
        },
      ]}
    />
  );
}
