import type { Metadata } from 'next';
import { StaticPageLayout } from '@/components/shared/StaticPageLayout';

export const metadata: Metadata = {
  title: 'Chính sách hoàn tiền · Refund Policy | VieLang',
  description: 'Chính sách hoàn tiền và huỷ buổi học tại VieLang.',
};

const UPDATED = '2026-07-04';

export default function RefundPage() {
  return (
    <StaticPageLayout
      updated={UPDATED}
      title={{ vn: 'Chính sách hoàn tiền', en: 'Refund Policy' }}
      intro={{
        vn: 'Chúng tôi muốn bạn yên tâm khi đặt buổi học. Dưới đây là quy tắc huỷ và hoàn tiền cho các buổi 1-on-1.',
        en: 'We want you to book with confidence. Below are the cancellation and refund rules for 1-on-1 sessions.',
      }}
      sections={[
        {
          heading: { vn: '1. Huỷ trước ≥ 24 giờ', en: '1. Cancel ≥ 24 hours before start' },
          paragraphs: [
            {
              vn: 'Bạn có thể huỷ hoặc đổi lịch một buổi 1-on-1 chậm nhất 24 giờ trước giờ bắt đầu và nhận hoàn tiền đầy đủ vào phương thức thanh toán ban đầu.',
              en: 'You may cancel or reschedule a 1-on-1 session up to 24 hours before start time for a full refund to your original payment method.',
            },
          ],
        },
        {
          heading: { vn: '2. Huỷ dưới 24 giờ', en: '2. Cancel within 24 hours' },
          paragraphs: [
            {
              vn: 'Huỷ trong vòng 24 giờ trước giờ bắt đầu không được hoàn tiền, vì giáo viên đã dành khung thời gian đó cho bạn. Trường hợp bất khả kháng (ốm đau, sự kiện đột xuất) có thể được xem xét cấp credit cho buổi khác — vui lòng liên hệ hỗ trợ.',
              en: 'Cancellations within 24 hours of start time are non-refundable, because the tutor has reserved that block for you. In exceptional circumstances (illness, emergencies) we may offer credit toward another session — please contact support.',
            },
          ],
        },
        {
          heading: { vn: '3. Giáo viên huỷ hoặc vắng mặt', en: '3. Tutor cancels or is a no-show' },
          paragraphs: [
            {
              vn: 'Nếu giáo viên huỷ hoặc không có mặt, bạn được hoàn 100% và chúng tôi sẽ hỗ trợ bạn tìm khung giờ khác với giáo viên đó hoặc giáo viên tương đương.',
              en: 'If the tutor cancels or does not attend, you receive a 100% refund and we will help you find another time with the same tutor or an equivalent one.',
            },
          ],
        },
        {
          heading: { vn: '4. Sự cố kỹ thuật', en: '4. Technical issues' },
          paragraphs: [
            {
              vn: 'Nếu buổi học không thể diễn ra do sự cố hạ tầng bên phía VieLang (video/audio không hoạt động, phòng học không tạo được), bạn nhận full credit hoặc hoàn tiền. Sự cố mạng cá nhân của bạn hoặc giáo viên không thuộc phạm vi này.',
              en: 'If a session cannot happen due to VieLang-side infrastructure issues (video/audio failure, room not provisioned), you receive full credit or a refund. Personal network issues on your side or the tutor’s side are not covered.',
            },
          ],
        },
        {
          heading: { vn: '5. Thời gian xử lý', en: '5. Processing time' },
          paragraphs: [
            {
              vn: 'Hoàn tiền được xử lý trong vòng 5–10 ngày làm việc kể từ khi được xác nhận. Thời gian nhận được có thể khác nhau tuỳ ngân hàng hoặc cổng thanh toán.',
              en: 'Refunds are processed within 5–10 business days of confirmation. Actual arrival time varies by bank or payment provider.',
            },
          ],
        },
        {
          heading: { vn: '6. Buổi group free-talk', en: '6. Group free-talk rooms' },
          paragraphs: [
            {
              vn: 'Các phòng group free-talk hiện miễn phí. Bạn có thể huỷ chỗ giữ bất cứ lúc nào trước giờ bắt đầu; không có giao dịch tiền để hoàn.',
              en: 'Group free-talk rooms are currently free. You can cancel your reservation any time before start; there is no transaction to refund.',
            },
          ],
        },
        {
          heading: { vn: '7. Câu hỏi', en: '7. Questions' },
          paragraphs: [
            {
              vn: 'Vấn đề về hoàn tiền, vui lòng liên hệ qua trang Liên hệ trong vòng 30 ngày kể từ buổi học.',
              en: 'For refund questions, please contact us via the Contact page within 30 days of the session.',
            },
          ],
        },
      ]}
    />
  );
}
