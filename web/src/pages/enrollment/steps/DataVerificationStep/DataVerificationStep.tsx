import './style.scss';

import { zodResolver } from '@hookform/resolvers/zod';
import { useEffect, useMemo, useRef } from 'react';
import { type SubmitHandler, useForm } from 'react-hook-form';
import { z } from 'zod';
import { shallow } from 'zustand/shallow';
import { useI18nContext } from '../../../../i18n/i18n-react';
import { FormInput } from '../../../../shared/components/Form/FormInput/FormInput';
import { Card } from '../../../../shared/components/layout/Card/Card';
import { MessageBox } from '../../../../shared/components/layout/MessageBox/MessageBox';
import { MessageBoxType } from '../../../../shared/components/layout/MessageBox/types';
import { EnrollmentStepIndicator } from '../../components/EnrollmentStepIndicator/EnrollmentStepIndicator';
import { useEnrollmentStore } from '../../hooks/store/useEnrollmentStore';

const phonePattern = /^\+?[0-9]+( [0-9]+)?$/;

export const DataVerificationStep = () => {
  const { LL } = useI18nContext();
  const submitRef = useRef<HTMLInputElement | null>(null);

  const nextSubject = useEnrollmentStore((state) => state.nextSubject);

  const userInfo = useEnrollmentStore((state) => state.userInfo);

  const [setEnrollment, next] = useEnrollmentStore(
    (state) => [state.setState, state.nextStep],
    shallow,
  );

  const pageLL = LL.pages.enrollment.steps.dataVerification;

  const schema = useMemo(
    () =>
      z.object({
        phone: z
          .string()
          .trim()
          .refine((val) => {
            if (val && typeof val === 'string' && val.length > 0) {
              return phonePattern.test(val);
            }
            return true;
          }, LL.form.errors.invalid())
          .regex(phonePattern, LL.form.errors.invalid()),
      }),
    [LL.form.errors],
  );

  type FormFields = z.infer<typeof schema>;

  const defaultValues = useMemo(() => {
    const res: FormFields = {
      phone: userInfo?.phone_number ?? '',
    };
    return res;
  }, [userInfo]);

  const { control, handleSubmit } = useForm<FormFields>({
    defaultValues: defaultValues,
    mode: 'all',
    resolver: zodResolver(schema),
  });

  const handleValidSubmit: SubmitHandler<FormFields> = (values) => {
    const phone = values.phone.length > 0 ? values.phone : undefined;
    if (userInfo) {
      setEnrollment({
        userInfo: { ...userInfo, phone_number: phone },
      });
      next();
    }
  };

  useEffect(() => {
    const sub = nextSubject.subscribe(() => {
      submitRef.current?.click();
    });

    return () => {
      sub.unsubscribe();
    };
  }, [nextSubject]);

  return (
    <Card id="enrollment-data-verification-card">
      <EnrollmentStepIndicator />
      <h3>{pageLL.title()}</h3>
      <MessageBox type={MessageBoxType.INFO} message={pageLL.messageBox()} />
      <form
        data-testid="enrollment-data-verification"
        onSubmit={handleSubmit(handleValidSubmit)}
      >
        <div className="row">
          <div className="item">
            <label>{pageLL.form.fields.firstName.label()}:</label>
            <p>{userInfo?.first_name}</p>
          </div>
          <div className="item">
            <label>{pageLL.form.fields.lastName.label()}:</label>
            <p>{userInfo?.last_name}</p>
          </div>
        </div>
        <div className="row">
          <div className="item">
            <label>{pageLL.form.fields.email.label()}:</label>
            <p>{userInfo?.email}</p>
          </div>
          <FormInput
            label={pageLL.form.fields.phone.label()}
            controller={{ control, name: 'phone' }}
          />
        </div>
        <input className="hidden" ref={submitRef} type="submit" />
      </form>
    </Card>
  );
};
