import { beforeEach, describe, expect, it, vi } from 'vitest';
import { Route } from './callback';

const mocks = vi.hoisted(() => ({
  callback: vi.fn(),
  setError: vi.fn(),
}));

vi.mock('../../shared/api/api', () => ({
  api: {
    openId: {
      enrollmentCallback: {
        callbackFn: mocks.callback,
      },
    },
    enrollment: {
      start: {
        callbackFn: vi.fn(),
      },
    },
  },
}));
vi.mock('../../paraglide/messages', () => ({
  m: {
    openid_generic_error: () => 'Sign-in failed',
  },
}));
vi.mock('../../shared/hooks/useEnrollmentStore', () => ({
  useEnrollmentStore: {
    setState: vi.fn(),
  },
}));
vi.mock('../../shared/hooks/useOpenIdStore', () => ({
  useOpenidStore: {
    setState: mocks.setError,
  },
}));

const beforeLoad = () =>
  (
    Route.options.beforeLoad as (args: {
      search: { code: string; state: string };
    }) => Promise<unknown>
  )({
    search: { code: 'code', state: 'state' },
  });

describe('OpenID enrollment callback route', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('shows a friendly error with the provider error as details', async () => {
    mocks.callback.mockRejectedValue({
      isAxiosError: true,
      response: {
        data: {
          error: 'invalid_client: The provided client secret is invalid.',
        },
      },
    });

    await expect(beforeLoad()).rejects.toMatchObject({
      options: { replace: true, to: '/openid/error' },
    });
    expect(mocks.setError).toHaveBeenCalledWith({
      error: 'Sign-in failed',
      details: 'invalid_client: The provided client secret is invalid.',
    });
  });

  it('shows no details when the failure has no error text', async () => {
    mocks.callback.mockRejectedValue(new Error('Network Error'));

    await expect(beforeLoad()).rejects.toMatchObject({
      options: { replace: true, to: '/openid/error' },
    });
    expect(mocks.setError).toHaveBeenCalledWith({
      error: 'Sign-in failed',
      details: undefined,
    });
  });
});
