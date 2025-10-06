import axios from 'axios';
import qs from 'qs';

const baseUrl = import.meta.env.VITE_UPDATE_BASE_URL as string | undefined;

const client = axios.create({
  baseURL: baseUrl ?? 'https://update-service-dev.defguard.net/api',
  headers: { 'Content-Type': 'application/json' },
  paramsSerializer: {
    serialize: (params) =>
      qs.stringify(params, {
        arrayFormat: 'repeat',
      }),
  },
});

export type ClientVersionCheck = {
  windows_amd64?: string;
  deb_amd64?: string;
  deb_arm64?: string;
  rpm_amd64?: string;
  rpm_arm64?: string;
  macos_amd64?: string;
  macos_arm64?: string;
};

const updateServiceApi = {
  getClientArtifacts: () =>
    client
      .get<ClientVersionCheck>('/update/artifacts', {
        params: {
          product: 'defguard-client',
          source: 'enrollment',
        },
      })
      .then((response) => response.data),
} as const;

export { updateServiceApi };
