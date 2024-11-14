import './style.scss';

import { useQuery } from '@tanstack/react-query';
import { useEffect, useState } from 'react';
import { useBreakpoint } from 'use-breakpoint';

import { AxiosError } from 'axios';
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
      const hashFragment = window.location.hash.substring(1);
      const params = new URLSearchParams(hashFragment);
      const error = params.get('error');
      if (error) {
        setError(error);
        return;
      }
      const id_token = params.get('id_token');
      const state = params.get('state');
      if (!id_token || !state) {
        setError(
          "Missing id_token or state in the callback's URL. The provider might not be configured correctly.",
        );
        return;
      }
      if (id_token && state) {
        return openIDCallback({
          id_token,
          state,
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
        console.log('errorResponse', errorResponse);
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
    <>
      <Card shaded={breakpoint !== 'mobile'} className="openidcallback-card">
        <h2>Start your enrollment process</h2>
        <BigInfoBox
          message={
            'Thank you for validating your account, please follow instruction below for configuring your VPN connection.'
          }
        />
        <div className="steps">
          <p>1. Please download and install defguard VPN Desktop Client</p>
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
          <p>
            2. Open the client and <i>Add Instance</i>. Copy the data provided below into
            the corresponding fields. You can also learn more about the process in our{' '}
            <a
              href="https://docs.defguard.net/help/configuring-vpn/add-new-instance"
              target="_blank"
            >
              documentation
            </a>
            .
          </p>
          <Card className="token-input shaded">
            <h2>Please provide instance URL and token</h2>
            <div className="labelled-input">
              <label>Instance URL:</label>
              <div className="input-copy">
                <Input value={data?.url} readOnly />
                <ActionButton
                  variant={ActionButtonVariant.COPY}
                  onClick={() => {
                    navigator.clipboard.writeText(data?.url);
                  }}
                />
              </div>
            </div>

            <div className="labelled-input">
              <label>Token:</label>
              <div className="input-copy">
                <Input value={data?.token} readOnly />
                <ActionButton
                  variant={ActionButtonVariant.COPY}
                  onClick={() => {
                    navigator.clipboard.writeText(data?.token);
                  }}
                />
              </div>
            </div>

            <Button
              text="Add instance"
              styleVariant={ButtonStyleVariant.PRIMARY}
              disabled={true}
            />
          </Card>
        </div>
      </Card>
    </>
  );
};
