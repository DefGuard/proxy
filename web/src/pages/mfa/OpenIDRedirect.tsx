import './style.scss';

import { LogoContainer } from '../../components/LogoContainer/LogoContainer';
import { LoaderSpinner } from '../../shared/components/layout/LoaderSpinner/LoaderSpinner';
import { PageContainer } from '../../shared/components/layout/PageContainer/PageContainer';
import { useApi } from '../../shared/hooks/api/useApi';
import { useQuery } from '@tanstack/react-query';

import { useEffect } from 'react';
import { AuthFailIcon } from './icons';
import { useI18nContext } from '../../i18n/i18n-react';

export const OpenIdMfaPage = () => {
  const { getOpenIDAuthInfo } = useApi();
  const { LL } = useI18nContext();
  const urlParams = new URLSearchParams(window.location.search);
  const token = urlParams.get('token');

  const { isLoading: openidLoading, data: openidData } = useQuery(
    [],
    () =>
      getOpenIDAuthInfo({
        state: token || undefined,
        type: 'mfa',
      }),
    {
      refetchOnMount: true,
      refetchOnWindowFocus: false,
      enabled: !!token,
    },
  );

  useEffect(() => {
    if (!openidLoading && openidData?.url) {
      window.location.href = openidData.url;
    }
  }, [openidLoading, openidData?.url]);

  return (
    <PageContainer id="openid-mfa-page">
      <LogoContainer />
      <div className="content">
        {!token && (
          <>
            <h1>{LL.pages.openidMfaRedirect.error.title()}</h1>
            <AuthFailIcon />
            <p>{LL.pages.openidMfaRedirect.error.message()}</p>
          </>
        )}
        {token && <LoaderSpinner size={100} />}
      </div>
    </PageContainer>
  );
};
