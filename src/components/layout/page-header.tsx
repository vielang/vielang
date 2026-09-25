/**
 * Phần đầu của các trang cấp một (Thư viện, Luyện thi, Cẩm nang, Góc học
 * tập): tiêu đề cùng cỡ và một dòng phụ.
 *
 * Trước đây mỗi trang một kiểu — Thư viện không có tiêu đề, Luyện thi có hai
 * tầng tiêu đề, Cẩm nang một tiêu đề nhỏ hơn — nên mỗi lần đổi tab bố cục
 * nhảy một kiểu, phải dò lại xem đang ở đâu.
 */
export function PageHeader({ title, subtitle }: { title: string; subtitle?: string }) {
  return (
    <header>
      <h1 className="text-2xl font-semibold tracking-tight">{title}</h1>
      {subtitle && <p className="mt-1 text-sm text-muted-foreground">{subtitle}</p>}
    </header>
  );
}

/**
 * Tên một nhóm trong trang (Cấp 1, Du học…): chữ đậm vừa, kèm phần phụ màu
 * nhạt (chữ Hàn, số bài). Một kiểu cho mọi trang — trước đây Thư viện dùng
 * tiêu đề có gạch chân, Cẩm nang dùng chữ in hoa nhỏ.
 */
export function SectionLabel({
  children,
  aside,
  id,
}: {
  children: React.ReactNode;
  aside?: React.ReactNode;
  id?: string;
}) {
  return (
    <h2 id={id} className="flex items-baseline gap-2 text-base font-semibold tracking-tight">
      {children}
      {aside !== undefined && aside !== null && aside !== "" && (
        <span className="text-sm font-normal text-muted-foreground">{aside}</span>
      )}
    </h2>
  );
}
