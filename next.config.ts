import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  redirects: async () => [
    // Danh sách khoá IT nay là trang chủ.
    { source: "/it", destination: "/", permanent: true },
    // Cẩm nang không có trang tổng: mở thẳng mục đầu tiên. Tạm thời (307) —
    // sau này có trang tổng thì trình duyệt không nhớ mãi đường chuyển cũ.
    { source: "/cam-nang", destination: "/cam-nang/lo-trinh", permanent: false },
  ],
};

export default nextConfig;
