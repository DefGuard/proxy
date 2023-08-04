import './style.scss';

import { zodResolver } from '@hookform/resolvers/zod';
import { useMutation } from '@tanstack/react-query';
import dayjs from 'dayjs';
import { useMemo } from 'react';
import { SubmitHandler, useForm } from 'react-hook-form';
import { useNavigate } from 'react-router-dom';
import { useBreakpoint } from 'use-breakpoint';
import { z } from 'zod';

import { useI18nContext } from '../../../i18n/i18n-react';
import { FormInput } from '../../../shared/components/Form/FormInput/FormInput';
import { ArrowSingle } from '../../../shared/components/icons/ArrowSingle/ArrowSingle';
import {
  ArrowSingleDirection,
  ArrowSingleSize,
} from '../../../shared/components/icons/ArrowSingle/types';
import { Button } from '../../../shared/components/layout/Button/Button';
import {
  ButtonSize,
  ButtonStyleVariant,
} from '../../../shared/components/layout/Button/types';
import { Card } from '../../../shared/components/layout/Card/Card';
import { MessageBox } from '../../../shared/components/layout/MessageBox/MessageBox';
import { MessageBoxType } from '../../../shared/components/layout/MessageBox/types';
import { deviceBreakpoints } from '../../../shared/constants';
import { useApi } from '../../../shared/hooks/api/useApi';
import { routes } from '../../../shared/routes';
import { useEnrollmentStore } from '../../enrollment/hooks/store/useEnrollmentStore';

type FormFields = {
  token: string;
};

export const TokenCard = () => {
  const navigate = useNavigate();
  const {
    enrollment: { start: startEnrollment },
  } = useApi();
  const { breakpoint } = useBreakpoint(deviceBreakpoints);
  const { LL } = useI18nContext();
  const initEnrollment = useEnrollmentStore((state) => state.init);
  const schema = useMemo(
    () =>
      z
        .object({
          token: z
            .string()
            .trim()
            .min(1, LL.pages.token.card.form.errors.token.required()),
        })
        .required(),
    [LL.pages.token.card.form.errors.token],
  );

  const { control, handleSubmit, setError } = useForm<FormFields>({
    mode: 'all',
    defaultValues: {
      token: '',
    },
    resolver: zodResolver(schema),
  });

  const { isLoading, mutate } = useMutation({
    mutationFn: startEnrollment,
    onSuccess: (res) => {
      const sessionEndDate = dayjs.unix(res.deadline_timestamp).toDate();
      initEnrollment({
        step: 0,
        userInfo: res.user,
        adminInfo: res.admin,
        sessionEnd: sessionEndDate.toISOString(),
        vpnOptional: res.vpn_setup_optional,
        endContent: res.final_page_content,
      });
      navigate(routes.enrollment, { replace: true });
    },
    onError: (err) => {
      setError(
        'token',
        {
          message: LL.form.errors.invalid(),
        },
        {
          shouldFocus: true,
        },
      );
      console.error(err);
    },
  });

  const handleValidSubmit: SubmitHandler<FormFields> = (values) => {
    if (!isLoading) {
      mutate({
        token: values.token,
      });
    }
  };

  return (
    <Card shaded={breakpoint !== 'mobile'} className="token-card">
      <h2>{LL.pages.token.card.title()}</h2>
      <MessageBox
        message={LL.pages.token.card.messageBox.email()}
        type={MessageBoxType.INFO}
        dismissId="token-page-card-email"
      />
      <form onSubmit={handleSubmit(handleValidSubmit)}>
        <FormInput
          controller={{ control, name: 'token' }}
          placeholder={LL.pages.token.card.form.fields.token.placeholder()}
          required
        />
        <div className="controls">
          <Button
            type="submit"
            loading={isLoading}
            size={ButtonSize.LARGE}
            styleVariant={ButtonStyleVariant.PRIMARY}
            text={LL.pages.token.card.form.controls.submit()}
            rightIcon={
              <ArrowSingle
                direction={ArrowSingleDirection.RIGHT}
                size={ArrowSingleSize.LARGE}
              />
            }
          />
        </div>
      </form>
    </Card>
  );
};
