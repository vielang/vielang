// UI copy for the two supported languages (VN + EN).
//
// Only keys actually referenced by `t(...)` calls in the app live here — the
// golf-era + chat-widget strings were pruned when their consumers were
// deleted. Active consumers: LoginPage, Header.
//
// If a key is missing for the active language, LangContext falls back to EN
// then to the raw key, so it is safe to add a key to only one language
// during a rollout — but ship the pair.

export type TranslationKey =
  | 'login'
  | 'logout'
  | 'signUp'
  | 'signUpButton'
  | 'noAccount'
  | 'registerNow'
  | 'alreadyHaveAccount'
  | 'loginNow'
  | 'forgotPassword'
  | 'passwordMismatch'
  | 'tutors'
  | 'news'
  | 'myPage'
  | 'menu'
  | 'language';

type Dict = Record<TranslationKey, string>;

const VN: Dict = {
  // Auth
  login: 'Đăng nhập',
  logout: 'Đăng xuất',
  signUp: 'Đăng ký',
  signUpButton: 'Đăng ký',
  noAccount: 'Chưa có tài khoản?',
  registerNow: 'Đăng ký ngay',
  alreadyHaveAccount: 'Đã có tài khoản?',
  loginNow: 'Đăng nhập ngay',
  forgotPassword: 'Quên mật khẩu?',
  passwordMismatch: 'Mật khẩu không khớp.',

  // Navigation
  tutors: 'Giáo viên',
  news: 'Tin tức',
  myPage: 'Trang cá nhân',
  menu: 'Menu',
  language: 'Ngôn ngữ',
};

const EN: Dict = {
  // Auth
  login: 'Log in',
  logout: 'Log out',
  signUp: 'Sign up',
  signUpButton: 'Sign up',
  noAccount: 'No account?',
  registerNow: 'Register now',
  alreadyHaveAccount: 'Already have an account?',
  loginNow: 'Log in',
  forgotPassword: 'Forgot password?',
  passwordMismatch: 'Passwords do not match.',

  // Navigation
  tutors: 'Tutors',
  news: 'News',
  myPage: 'My page',
  menu: 'Menu',
  language: 'Language',
};

export const translations: Record<'VN' | 'EN', Dict> = { VN, EN };
