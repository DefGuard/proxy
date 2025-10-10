import { z } from 'zod';
import { m } from '../../paraglide/messages';
import { Container } from '../../shared/components/Container/Container';
import { Page } from '../../shared/components/Page/Page';
import { SizedBox } from '../../shared/defguard-ui/components/SizedBox/SizedBox';
import { ThemeSpacing } from '../../shared/defguard-ui/types';

import './style.scss';
import { useMutation } from '@tanstack/react-query';
import { useNavigate } from '@tanstack/react-router';
import { api } from '../../shared/api/api';
import { PageNavigation } from '../../shared/components/PageNavigation/PageNavigation';
import { useAppForm } from '../../shared/defguard-ui/form';

const {
  password: { sendEmail },
} = api;

const formSchema = z.object({
  email: z.email(m.form_error_email()).trim().min(1, m.form_error_required()),
});

type FormFields = z.infer<typeof formSchema>;

const defaults: FormFields = {
  email: '',
};

export const PasswordStartPage = () => {
  const navigate = useNavigate();

  const { mutateAsync } = useMutation({
    mutationFn: sendEmail.callbackFn,
    onError: (e) => {
      console.error('Error while sending email.', e);
    },
  });

  const form = useAppForm({
    defaultValues: defaults,
    validators: {
      onChange: formSchema,
      onSubmit: formSchema,
    },
    onSubmit: async ({ value }) => {
      await mutateAsync({
        data: {
          email: value.email,
        },
      });
      navigate({
        to: '/password/sent',
        replace: true,
      });
    },
  });

  return (
    <Page id="password-start-page" variant="default" nav>
      <header>
        <h1>{m.password_start_title()}</h1>
        <p>{m.password_start_subtitle()}</p>
      </header>
      <SizedBox height={ThemeSpacing.Xl3} />
      <Container>
        <p>{m.password_start_form_explain()}</p>
        <form.AppForm>
          <form.AppField name="email">
            {(field) => (
              <field.FormInput label={m.password_start_form_label_email()} required />
            )}
          </form.AppField>
        </form.AppForm>
      </Container>
      <PageNavigation
        loading={form.state.isSubmitting}
        backText={m.controls_back()}
        nextText={m.password_start_submit()}
        onBack={() => {
          navigate({
            to: '/',
            replace: true,
          });
        }}
        onNext={() => {
          form.handleSubmit();
        }}
      />
    </Page>
  );
};
