import { z } from 'zod';
import { m } from '../../paraglide/messages';
import { Container } from '../../shared/components/Container/Container';
import { Page } from '../../shared/components/Page/Page';
import { EnrollmentStep } from '../../shared/components/Step/Step';
import { SizedBox } from '../../shared/defguard-ui/components/SizedBox/SizedBox';
import { ThemeSpacing } from '../../shared/defguard-ui/types';

import './style.scss';
import { useNavigate } from '@tanstack/react-router';
import { PageNavigation } from '../../shared/components/PageNavigation/PageNavigation';
import { useAppForm } from '../../shared/defguard-ui/form';

const formSchema = z.object({
  email: z.email(m.form_error_email()).trim().min(1, m.form_error_required()),
});

type FormFields = z.infer<typeof formSchema>;

const defaults: FormFields = {
  email: '',
};

export const PasswordStartPage = () => {
  const navigate = useNavigate();

  const form = useAppForm({
    defaultValues: defaults,
    validators: {
      onChange: formSchema,
      onSubmit: formSchema,
    },
    onSubmit: () => {
      navigate({
        to: '/password/sent',
        replace: true,
      });
    },
  });

  return (
    <Page id="password-start-page" variant="default" nav>
      <EnrollmentStep current={0} max={1} />
      <header>
        <h1>{m.password_start_title()}</h1>
        <p>{m.password_start_subtitle()}</p>
      </header>
      <SizedBox height={ThemeSpacing.Xl5} />
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
