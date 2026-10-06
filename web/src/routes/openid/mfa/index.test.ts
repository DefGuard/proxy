import { beforeEach, describe, expect, it, vi } from 'vitest';
import { Route } from './index';

const mocks = vi.hoisted(() => ({
  authInfo: vi.fn(),
}));

vi.mock('../../../shared/api/api', () => ({
  api: {
    openId: {
      authInfo: {
        callbackFn: mocks.authInfo,
      },
    },
  },
}));

const validateSearch = (input: unknown) => {
  const validator = Route.options.validateSearch as unknown as
    | ((value: unknown) => unknown)
    | { parse: (value: unknown) => unknown };
  return typeof validator === 'function' ? validator(input) : validator.parse(input);
};

const load = (search: unknown) =>
  (Route.options.loader as (args: { deps: { search: unknown } }) => Promise<unknown>)({
    deps: { search },
  });

describe('OpenID MFA entry route', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.authInfo.mockResolvedValue({ data: { url: 'https://idp.example/authorize' } });
  });

  it('accepts legacy token-only URLs', async () => {
    const token = 'legacy-token';
    const search = validateSearch({ token });

    await expect(load(search)).resolves.toBe('https://idp.example/authorize');
    expect(mocks.authInfo).toHaveBeenCalledWith({
      data: {
        state: token,
        type: 'mfa',
      },
    });
  });

  it('passes token and step attempt ID to Core as composite state', async () => {
    const token = 'mfa-token';
    const stepAttemptId = 'attempt-1';
    const search = validateSearch({ token, step_attempt_id: stepAttemptId });

    await load(search);

    expect(mocks.authInfo).toHaveBeenCalledWith({
      data: {
        state: `${token}.${stepAttemptId}`,
        type: 'mfa',
      },
    });
  });
});
