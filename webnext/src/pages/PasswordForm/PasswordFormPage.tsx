import z from 'zod';
import { m } from '../../paraglide/messages';
import { Container } from '../../shared/components/Container/Container';
import { Page } from '../../shared/components/Page/Page';
import './style.scss';
import { revalidateLogic, useStore } from '@tanstack/react-form';
import { useNavigate } from '@tanstack/react-router';
import clsx from 'clsx';
import { useCallback } from 'react';
import { Button } from '../../shared/defguard-ui/components/Button/Button';
import { Icon } from '../../shared/defguard-ui/components/Icon';
import type { IconKindValue } from '../../shared/defguard-ui/components/Icon/icon-types';
import { SizedBox } from '../../shared/defguard-ui/components/SizedBox/SizedBox';
import { useAppForm, withForm } from '../../shared/defguard-ui/form';
import { ThemeSpacing } from '../../shared/defguard-ui/types';

const PasswordErrorCode = {
  Number: 'password_form_check_number',
  Special: 'password_form_check_special',
  Lowercase: 'password_form_check_lowercase',
  Uppercase: 'password_form_check_uppercase',
  Minimum: 'password_form_check_minimum',
} as const;

const errorCodes = Object.values(PasswordErrorCode);

type PasswordErrorCodeValue = (typeof PasswordErrorCode)[keyof typeof PasswordErrorCode];

const errorIsCustomCode = (value: string): value is PasswordErrorCodeValue => {
  return (errorCodes as readonly string[]).includes(value);
};

const mapPasswordFieldError = (errorValue: string): string => {
  if (errorIsCustomCode(errorValue)) {
    return m.password_form_special_error();
  }
  return errorValue;
};

const hasNumber = /[0-9]/;

const hasUppercase = /[A-Z]/;

const hasLowercase = /[a-z]/;

const hasSpecialChar = /[^a-zA-Z0-9]/;

const formSchema = z
  .object({
    password: z.string().trim().min(1, m.form_error_required()),
    repeat: z.string().trim().min(1, m.form_error_required()),
  })
  .superRefine(({ password, repeat }, ctx) => {
    if (password.length < 8) {
      ctx.addIssue({
        message: PasswordErrorCode.Minimum,
        code: 'custom',
        path: ['password'],
        continue: true,
      });
    }
    if (!hasNumber.test(password)) {
      ctx.addIssue({
        message: PasswordErrorCode.Number,
        code: 'custom',
        path: ['password'],
        continue: true,
      });
    }
    if (!hasUppercase.test(password)) {
      ctx.addIssue({
        message: PasswordErrorCode.Uppercase,
        code: 'custom',
        path: ['password'],
        continue: true,
      });
    }
    if (!hasLowercase.test(password)) {
      ctx.addIssue({
        message: PasswordErrorCode.Lowercase,
        code: 'custom',
        continue: true,
        path: ['password'],
      });
    }
    if (!hasSpecialChar.test(password)) {
      ctx.addIssue({
        message: PasswordErrorCode.Special,
        code: 'custom',
        continue: true,
        path: ['password'],
      });
    }
    if (repeat.length) {
      ctx.addIssue({
        message: m.password_form_check_repeat_match(),
        code: 'custom',
        path: ['repeat'],
        continue: true,
      });
    }
  });

type FormFields = z.infer<typeof formSchema>;

const defaultFormValues: FormFields = {
  password: '',
  repeat: '',
};

export const PasswordFormPage = () => {
  const navigate = useNavigate();

  const form = useAppForm({
    defaultValues: defaultFormValues,
    validationLogic: revalidateLogic({
      mode: 'change',
      modeAfterSubmission: 'change',
    }),
    validators: {
      onDynamic: formSchema,
    },
    onSubmit: (values) => {
      console.table(values);
      navigate({
        to: '/password/form/finish',
        replace: true,
      });
    },
  });

  return (
    <Page id="password-reset-form-page" variant="small">
      <header>
        <h1>{m.password_form_title()}</h1>
        <p>{m.password_form_subtitle()}</p>
      </header>
      <SizedBox height={ThemeSpacing.Xl5} />
      <Container>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            e.stopPropagation();
          }}
        >
          <form.AppForm>
            <form.AppField name="password">
              {(field) => (
                <field.FormInput
                  mapError={mapPasswordFieldError}
                  label={m.password_form_labels_password()}
                  required
                />
              )}
            </form.AppField>
            <form.AppField
              name="repeat"
              validators={{
                onChangeListenTo: ['password'],
              }}
            >
              {(field) => (
                <field.FormInput label={m.password_form_labels_repeat()} required />
              )}
            </form.AppField>
            <CheckList form={form} />
            <Button size="big" type="submit" text={m.password_form_submit()} />
          </form.AppForm>
        </form>
      </Container>
    </Page>
  );
};

const CheckList = withForm({
  defaultValues: defaultFormValues,
  render: ({ form }) => {
    const passwordFieldErrors = useStore(
      form.store,
      (state) =>
        (state.fieldMeta.password?.errors as z.core.$ZodIssue[])
          ?.filter((issue) => issue.code === 'custom')
          .map((issue) => issue.message) ?? [],
    );

    const isPasswordClean = useStore(
      form.store,
      (state) => state.fieldMeta.password?.isPristine ?? true,
    );

    const isChecked = useCallback(
      (value: PasswordErrorCodeValue): boolean =>
        !passwordFieldErrors.includes(value) && !isPasswordClean,
      [passwordFieldErrors, isPasswordClean],
    );

    return (
      <div className="checklist">
        <p>{m.password_form_check_title()}</p>
        <ul>
          <CheckListItem
            errorCode="password_form_check_minimum"
            checked={isChecked('password_form_check_minimum')}
          />
          <CheckListItem
            errorCode="password_form_check_number"
            checked={isChecked('password_form_check_number')}
          />
          <CheckListItem
            errorCode="password_form_check_special"
            checked={isChecked('password_form_check_special')}
          />
          <CheckListItem
            errorCode="password_form_check_lowercase"
            checked={isChecked('password_form_check_lowercase')}
          />
          <CheckListItem
            errorCode="password_form_check_uppercase"
            checked={isChecked('password_form_check_uppercase')}
          />
        </ul>
      </div>
    );
  },
});

const CheckListItem = ({
  checked,
  errorCode,
}: {
  errorCode: PasswordErrorCodeValue;
  checked: boolean;
}) => {
  const iconKind: IconKindValue = checked ? 'check-filled' : 'empty-point';

  return (
    <li
      className={clsx({
        active: checked,
      })}
    >
      <Icon icon={iconKind} size={16} /> <span>{m[errorCode]()}</span>
    </li>
  );
};
