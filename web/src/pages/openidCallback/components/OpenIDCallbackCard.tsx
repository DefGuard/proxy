import './style.scss';

import { useQuery } from '@tanstack/react-query';
import type { AxiosError } from 'axios';
import parse from 'html-react-parser';
import { useState } from 'react';
import { useBreakpoint } from 'use-breakpoint';

import { useI18nContext } from '../../../i18n/i18n-react';
import { ActionButton } from '../../../shared/components/layout/ActionButton/ActionButton';
import { ActionButtonVariant } from '../../../shared/components/layout/ActionButton/types';
import { BigInfoBox } from '../../../shared/components/layout/BigInfoBox/BigInfoBox';
import { Button } from '../../../shared/components/layout/Button/Button';
import { ButtonStyleVariant } from '../../../shared/components/layout/Button/types';
import { Card } from '../../../shared/components/layout/Card/Card';
import { Input } from '../../../shared/components/layout/Input/Input';
import { LoaderSpinner } from '../../../shared/components/layout/LoaderSpinner/LoaderSpinner';
import { MessageBox } from '../../../shared/components/layout/MessageBox/MessageBox';
import { MessageBoxType } from '../../../shared/components/layout/MessageBox/types';
import SvgIconDownload from '../../../shared/components/svg/IconDownload';
import { deviceBreakpoints } from '../../../shared/constants';
import { useApi } from '../../../shared/hooks/api/useApi';

type ErrorResponse = {
  error: string;
};

export const OpenIDCallbackCard = () => {
  const { openIDCallback } = useApi();
  const { breakpoint } = useBreakpoint(deviceBreakpoints);
  const { LL } = useI18nContext();
  const [error, setError] = useState<string | null>(null);

  const { isLoading, data } = useQuery(
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
        return openIDCallback({
          code,
          state,
          type: 'enrollment',
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
      onSuccess(data) {
        if (!data?.token || !data?.url) {
          setError("The server's response is missing the token or url.");
        }
      },
    },
  );

  if (isLoading) {
    return (
      <div className="loader-container">
        <LoaderSpinner size={100} />
      </div>
    );
  }

  if (error) {
    return (
      <Card shaded={breakpoint !== 'mobile'} className="openidcallback-card">
        <MessageBox
          type={MessageBoxType.ERROR}
          message={`There was an error while validating your account: ${error}`}
        />
        <Button
          text="Go back"
          styleVariant={ButtonStyleVariant.PRIMARY}
          onClick={() => {
            window.location.href = '/';
          }}
        />
      </Card>
    );
  }

  return (
    <Card shaded={breakpoint !== 'mobile'} className="openidcallback-card">
      <h2>{LL.pages.oidcLogin.card.title()}</h2>
      <BigInfoBox message={LL.pages.oidcLogin.card.infoBox()} />
      <div className="steps">
        <p>1. {LL.pages.oidcLogin.card.steps.first()}</p>
        <div className="download-link">
          <Button
            text="Download Desktop Client"
            styleVariant={ButtonStyleVariant.PRIMARY}
            icon={<SvgIconDownload color="#fff" />}
            onClick={() => {
              window.open('https://defguard.net/download/', '_blank');
            }}
          />
        </div>
        <p>{parse(LL.pages.oidcLogin.card.steps.second())}</p>
        <Card className="token-input shaded">
          <h2>{LL.pages.oidcLogin.card.steps.tokenInput.title()}</h2>
          <div className="labelled-input">
            <label>{LL.pages.oidcLogin.card.steps.tokenInput.instanceUrl()}</label>
            <div className="input-copy">
              <Input value={data?.url} readOnly />
              <ActionButton
                variant={ActionButtonVariant.COPY}
                onClick={() => {
                  // This should never be undefined, but just in case
                  navigator.clipboard.writeText(data?.url || '');
                }}
              />
            </div>
          </div>

          <div className="labelled-input">
            <label>{LL.pages.oidcLogin.card.steps.tokenInput.token()}</label>
            <div className="input-copy">
              <Input value={data?.token} readOnly />
              <ActionButton
                variant={ActionButtonVariant.COPY}
                onClick={() => {
                  // This should never be undefined, but just in case
                  navigator.clipboard.writeText(data?.token || '');
                }}
              />
            </div>
          </div>

          <Button
            text={LL.pages.oidcLogin.card.steps.tokenInput.addInstance()}
            styleVariant={ButtonStyleVariant.PRIMARY}
            disabled={true}
          />
        </Card>
      </div>
    </Card>
  );
};
