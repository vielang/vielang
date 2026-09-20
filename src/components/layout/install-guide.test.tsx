import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { InstallGuide } from "./install-guide";
import { detectPlatform, useInstallStore } from "@/lib/install-store";

const UA = {
  iphone:
    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Safari/604.1",
  ipad: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 Safari/605.1.15",
  mac: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 Chrome/120",
  android: "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 Chrome/120 Mobile",
};

describe("nhận diện thiết bị", () => {
  it("iPhone", () => {
    expect(detectPlatform(UA.iphone, 5)).toBe("ios");
  });

  it("iPad đời mới — khai user-agent y hệt máy Mac", () => {
    // Phân biệt bằng số điểm chạm, vì Mac thật không có màn cảm ứng. Nhầm
    // chỗ này là iPad nhận hướng dẫn của máy tính, làm theo không ra.
    expect(detectPlatform(UA.ipad, 5)).toBe("ios");
  });

  it("Mac thật thì vẫn là máy tính", () => {
    expect(detectPlatform(UA.mac, 0)).toBe("desktop");
  });

  it("Android", () => {
    expect(detectPlatform(UA.android, 5)).toBe("android");
  });
});

function stubDevice(userAgent: string, maxTouchPoints = 0) {
  vi.stubGlobal("navigator", { userAgent, maxTouchPoints });
}

beforeEach(() => {
  useInstallStore.setState({ event: null, installed: false });
  stubDevice(UA.android, 5);
});

afterEach(() => vi.unstubAllGlobals());

describe("hướng dẫn", () => {
  it("mở sẵn đúng hệ điều hành đang dùng", () => {
    stubDevice(UA.iphone, 5);
    render(<InstallGuide />);

    expect(screen.getByRole("tab", { name: "iPhone / iPad" })).toHaveProperty(
      "ariaSelected",
      "true"
    );
    expect(screen.getByText(/Bấm nút Chia sẻ/)).toBeTruthy();
  });

  it("cảnh báo đúng chỗ hay vấp nhất trên iOS", () => {
    // Mở bằng Chrome trên iPhone thì không có mục "Thêm vào MH chính".
    stubDevice(UA.iphone, 5);
    render(<InstallGuide />);

    expect(screen.getByText(/Phải mở bằng Safari/)).toBeTruthy();
  });

  it("vẫn xem được hướng dẫn của máy khác", () => {
    // Hay gặp: mở trên máy tính rồi làm theo trên điện thoại.
    stubDevice(UA.mac, 0);
    render(<InstallGuide />);
    fireEvent.click(screen.getByRole("tab", { name: "Android" }));

    expect(screen.getByText(/Bấm nút ba chấm/)).toBeTruthy();
  });
});

describe("khi trình duyệt cài được bằng một nút", () => {
  const prompt = vi.fn().mockResolvedValue(undefined);

  function withPrompt() {
    useInstallStore.setState({
      event: {
        prompt,
        userChoice: Promise.resolve({ outcome: "accepted" }),
      } as never,
    });
  }

  it("đưa nút cài lên trước hướng dẫn", () => {
    withPrompt();
    render(<InstallGuide />);

    expect(screen.getByText("Cài ứng dụng")).toBeTruthy();
  });

  it("bấm thì gọi lời mời thật của trình duyệt", async () => {
    withPrompt();
    render(<InstallGuide />);
    fireEvent.click(screen.getByText("Cài ứng dụng"));
    await vi.waitFor(() => expect(prompt).toHaveBeenCalled());
  });

  it("dùng xong thì bỏ nút đi", async () => {
    // Lời mời của Chrome chỉ dùng được một lần — để nút lại thì bấm vào
    // chẳng ra gì.
    withPrompt();
    render(<InstallGuide />);
    fireEvent.click(screen.getByText("Cài ứng dụng"));

    await vi.waitFor(() => expect(useInstallStore.getState().event).toBeNull());
  });

  it("không có lời mời thì chỉ còn hướng dẫn", () => {
    render(<InstallGuide />);
    expect(screen.queryByText("Cài ứng dụng")).toBeNull();
  });
});

describe("khi đã cài rồi", () => {
  it("nói thẳng là đã cài, không bày lại các bước", () => {
    useInstallStore.setState({ installed: true });
    render(<InstallGuide />);

    expect(screen.getByText("Bạn đang dùng bản đã cài")).toBeTruthy();
    expect(screen.queryByRole("tab")).toBeNull();
  });
});
