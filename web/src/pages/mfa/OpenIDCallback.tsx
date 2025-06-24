import './style.scss';

import { LogoContainer } from '../../components/LogoContainer/LogoContainer';
import { PageContainer } from '../../shared/components/layout/PageContainer/PageContainer';
import { useApi } from '../../shared/hooks/api/useApi';
import { useQuery } from '@tanstack/react-query';

import { useState } from 'react';
import { useI18nContext } from '../../i18n/i18n-react';
import { AxiosError } from 'axios';
import { LoaderSpinner } from '../../shared/components/layout/LoaderSpinner/LoaderSpinner';
import { AuthFailIcon, ClientReturnIcon } from './icons';
import ReactMarkdown from 'react-markdown';
import rehypeSanitize from 'rehype-sanitize';

type ErrorResponse = {
  error: string;
};

export const OpenIdMfaCallbackPage = () => {
  const { openIDMFACallback } = useApi();
  const { LL } = useI18nContext();
  const [error, setError] = useState<string | null>(null);

  const { isLoading } = useQuery(
    [],
    () => {
      const params = new URLSearchParams(window.location.search);
      const error = params.get('error');
      if (error) {
        setError(error);
        return;
      }
      const code = params.get('code');
      const state = params.get('state');
      if (!code || !state) {
        setError(
          "Missing code or state in the callback's URL. \
          The provider might not be configured correctly.",
        );
        return;
      }
      if (code && state) {
        return openIDMFACallback({
          code,
          state,
          type: 'mfa',
        });
      }
    },
    {
      retry: false,
      refetchOnWindowFocus: false,
      refetchOnMount: false,
      onError: (error: AxiosError) => {
        console.error(error);
        const errorResponse = error.response?.data as ErrorResponse;
        if (errorResponse.error) {
          setError(errorResponse.error);
        } else {
          setError(String(error));
        }
      },
    },
  );

  return (
    <PageContainer id="openid-mfa-page">
      <LogoContainer />
      <div className="content">
        {isLoading ? (
          <LoaderSpinner size={100} />
        ) : error ? (
          <>
            <h1>{LL.pages.openidMfaCallback.error.title()}</h1>
            <AuthFailIcon />
            <ReactMarkdown rehypePlugins={[rehypeSanitize]}>
              {LL.pages.openidMfaCallback.error.message()}
            </ReactMarkdown>
            <div>
              <h2>{LL.pages.openidMfaCallback.error.detailsTitle()}</h2>
              <pre className="error-details">{error}</pre>
            </div>
          </>
        ) : (
          <>
            <h1>{LL.pages.openidMfaCallback.success.title()}</h1>
            <ClientReturnIcon />
            <ReactMarkdown rehypePlugins={[rehypeSanitize]}>
              {LL.pages.openidMfaCallback.success.message()}
            </ReactMarkdown>
          </>
        )}
      </div>
    </PageContainer>
  );
};
