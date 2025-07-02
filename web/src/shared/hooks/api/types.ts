export type EmptyApiResponse = Record<string, never>;

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
};

export type EnrollmentStartRequest = {
  token: string;
};

export type EnrollmentSettings = {
  vpn_setup_optional: boolean;
  only_client_activation: boolean;
  admin_device_management: boolean;
};

export type EnrollmentStartResponse = {
  admin: AdminInfo;
  user: UserInfo;
  deadline_timestamp: number;
  final_page_content: string;
  settings: EnrollmentSettings;
};

export type ActivateUserRequest = {
  phone_number?: string;
  password: string;
};

export type CreateDeviceRequest = {
  name: string;
  pubkey: string;
};

export type Device = {
  id: number;
  name: string;
  pubkey: string;
  // stored by frontend only
  privateKey?: string;
  user_id: number;
  // timestamp
  created_at: number;
};

export type DeviceConfig = {
  network_id: number;
  network_name: string;
  config: string;
};

export type CreateDeviceResponse = {
  device: Device;
  configs: DeviceConfig[];
};

export type AppInfo = {
  version: string;
};

export type PasswordResetStartRequest = {
  email: string;
};

export type PasswordResetStartResponse = {
  admin: AdminInfo;
  user: UserInfo;
  deadline_timestamp: number;
};

export type PasswordResetRequest = {
  password: string;
};

export type UseApi = {
  enrollment: {
    start: (data: EnrollmentStartRequest) => Promise<EnrollmentStartResponse>;
    activateUser: (data: ActivateUserRequest) => Promise<EmptyApiResponse>;
    createDevice: (data: CreateDeviceRequest) => Promise<CreateDeviceResponse>;
  };
  passwordReset: {
    request: (data: PasswordResetStartRequest) => Promise<EmptyApiResponse>;
    start: (data: EnrollmentStartRequest) => Promise<PasswordResetStartResponse>;
    reset: (data: PasswordResetRequest) => Promise<EmptyApiResponse>;
  };
  getAppInfo: () => Promise<AppInfo>;
  getOpenIDAuthInfo: (data: {
    state?: string;
    type: 'enrollment' | 'mfa';
  }) => Promise<{ url: string; button_display_name: string }>;
  openIDCallback: (data: { code: string; state: string; type: 'enrollment' }) => Promise<{
    token: string;
    url: string;
  }>;
  openIDMFACallback: (data: {
    code: string;
    state: string;
    type: 'mfa';
  }) => Promise<EmptyApiResponse>;
};
