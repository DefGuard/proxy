import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { Route } from './callback';

const mocks = vi.hoisted(() => ({
  callback: vi.fn(),
  setError: vi.fn(),
}));

vi.mock('../../../shared/api/api', () => ({
  api: {
    openId: {
      mfaCallback: {
        callbackFn: mocks.callback,
      },
    },
  },
}));
vi.mock('../../../paraglide/messages', () => ({
  m: {
    openid_mfa_redirect_error_message: () => 'MFA callback error',
    openid_mfa_redirect_error_missing_args: () => 'Missing callback arguments',
  },
}));
vi.mock('../../../shared/components/PageInfo/PageInfo', () => ({
  PageInfo: () => null,
}));
vi.mock('../../../shared/hooks/useOpenIdStore', () => ({
  useOpenidStore: {
    setState: mocks.setError,
  },
}));

const load = (search: { code: string; state: string }) =>
  (
    Route.options.loader as (args: {
      deps: { search: typeof search };
    }) => Promise<unknown>
  )({
    deps: { search },
  });

describe('OpenID MFA callback route', () => {
  let consoleError: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    vi.clearAllMocks();
    mocks.callback.mockRejectedValue({
      isAxiosError: true,
      response: { data: { error: 'provider rejected' } },
    });
    consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    consoleError.mockRestore();
  });

  it('preserves successful authentication responses', async () => {
    mocks.callback.mockResolvedValue({ data: {} });

    await expect(load({ code: 'code', state: 'state' })).resolves.toBeUndefined();
    expect(mocks.setError).not.toHaveBeenCalled();
  });

  it('redirects rejected authentication responses with the MFA error', async () => {
    await expect(load({ code: 'code', state: 'state' })).rejects.toMatchObject({
      options: {
        replace: true,
        to: '/openid/error',
      },
    });
    expect(mocks.setError).toHaveBeenCalledWith({ error: 'MFA callback error' });
  });

  it('uses the MFA error when the response has no error message', async () => {
    mocks.callback.mockRejectedValue({
      isAxiosError: true,
      response: { data: { error: '' } },
    });

    await expect(load({ code: 'code', state: 'state' })).rejects.toMatchObject({
      options: {
        replace: true,
        to: '/openid/error',
      },
    });
    expect(mocks.setError).toHaveBeenCalledWith({ error: 'MFA callback error' });
  });

  it('uses the MFA error for non-Axios callback failures', async () => {
    mocks.callback.mockRejectedValue(undefined);

    await expect(load({ code: 'code', state: 'state' })).rejects.toMatchObject({
      options: {
        replace: true,
        to: '/openid/error',
      },
    });
    expect(mocks.setError).toHaveBeenCalledWith({ error: 'MFA callback error' });
  });
});
