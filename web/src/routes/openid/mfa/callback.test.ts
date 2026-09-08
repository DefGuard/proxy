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
    openid_generic_error: () => 'OpenID authentication error',
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
      response: { data: { error: 'provider rejected' } },
    });
    consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    consoleError.mockRestore();
  });

  it('redirects rejected authentication responses to the error route', async () => {
    await expect(load({ code: 'code', state: 'state' })).rejects.toMatchObject({
      options: {
        replace: true,
        to: '/openid/error',
      },
    });
    expect(mocks.setError).toHaveBeenCalledWith({ error: 'provider rejected' });
  });
});
