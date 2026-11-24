import { afterEach, expect, it, vi } from 'vitest';
import { updateServiceApi, updateServiceClient } from './update-service';

type GetResponse = Awaited<ReturnType<typeof updateServiceClient.get>>;

afterEach(() => {
  vi.restoreAllMocks();
});

it('getClientArtifacts returns windows_arm64 from the update service and falls back to the download page when the key is missing or the request fails', async () => {
  const get = vi.spyOn(updateServiceClient, 'get');

  get.mockResolvedValueOnce({
    data: { windows_arm64: 'https://example.com/client_arm64.msi' },
  } as GetResponse);
  await expect(updateServiceApi.getClientArtifacts()).resolves.toMatchObject({
    windows_arm64: 'https://example.com/client_arm64.msi',
  });

  get.mockResolvedValueOnce({ data: {} } as GetResponse);
  await expect(updateServiceApi.getClientArtifacts()).resolves.toMatchObject({
    windows_arm64: 'https://defguard.net/download',
  });

  vi.spyOn(console, 'error').mockImplementation(() => {});
  get.mockRejectedValueOnce(new Error('network error'));
  await expect(updateServiceApi.getClientArtifacts()).resolves.toMatchObject({
    windows_arm64: 'https://defguard.net/download',
  });
});
