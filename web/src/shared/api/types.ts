import type { AxiosError } from 'axios';

export type EmptyApiResponse = Record<never, never>;

export type ErrorResponse = AxiosError<{
  message?: string;
  msg?: string;
}>;

export type TokenRequest = {
  token: string;
};

export type AdminInfo = {
  name: string;
  email: string;
  phone_number?: string;
};

export type UserInfo = {
  first_name: string;
  last_name: string;
  login: string;
  email: string;
  phone_number?: string;
  is_admin: boolean;
  is_active: boolean;
  enrolled: boolean;
};

export type AppInfo = {
  version: string;
};

export type EnrollmentSettings = {
  vpn_setup_optional: boolean;
  only_client_activation: boolean;
  admin_device_management: boolean;
};

export type EnrollmentStartResponse = {
  admin: AdminInfo;
  user: UserInfo;
  instance: InstanceInfo;
  deadline_timestamp: number;
  final_page_content: string;
  settings: EnrollmentSettings;
};

export type PasswordResetStartRequest = {
  email: string;
};

export type PasswordResetStartResponse = {
  deadline_timestamp: number;
};

export type PasswordResetFinishRequest = {
  password: string;
  token: string;
};

export type OpenIdType = 'enrollment' | 'mfa';

export type OpenIdAuthInfoRequest = {
  state?: string;
  type: OpenIdType;
};

export type OpenIdAuthInfoResponse = {
  url?: string;
  button_display_name?: string;
};

export type OpenIdCallbackRequest = {
  code: string;
  state: string;
  type: 'enrollment';
};

export type OpenIdCallbackResponse = {
  token: string;
  url: string;
};

export type OpenIdMfaCallbackRequest = {
  code: string;
  state: string;
  type: 'mfa';
};

export type InstanceInfo = {
  id: string;
  name: string;
  url: string;
  proxy_url: string;
  username: string;
  enterprise_enabled: boolean;
  disable_all_traffic: boolean;
  openid_display_name: string;
};
