import { useNavigate } from '@tanstack/react-router';
import { m } from '../../../paraglide/messages';
import { ContainerWithIcon } from '../../../shared/components/ContainerWithIcon/ContainerWithIcon';
import { Page } from '../../../shared/components/Page/Page';
import { PageNavigation } from '../../../shared/components/PageNavigation/PageNavigation';
import { EnrollmentStep } from '../../../shared/components/Step/Step';
import { Divider } from '../../../shared/defguard-ui/components/Divider/Divider';
import './style.scss';
import { revalidateLogic } from '@tanstack/react-form';
import z from 'zod';
import { SizedBox } from '../../../shared/defguard-ui/components/SizedBox/SizedBox';
import { useAppForm } from '../../../shared/defguard-ui/form';
import { ThemeSpacing } from '../../../shared/defguard-ui/types';

const formSchema = z.object({
  token: z.string().trim().min(1, m.form_error_required()),
});

type FormFields = z.infer<typeof formSchema>;

const defaultValues: FormFields = {
  token: '',
};

export const EnrollmentStartPage = () => {
  const navigate = useNavigate();

  const form = useAppForm({
    defaultValues,
    validationLogic: revalidateLogic({
      mode: 'change',
      modeAfterSubmission: 'change',
    }),
    validators: {
      onDynamic: formSchema,
      onSubmit: formSchema,
    },
    onSubmit: (values) => {
      console.table(values);
      navigate({
        to: '/client-setup',
        replace: true,
      });
    },
    onSubmitInvalid: (props) => {
      console.log(props);
    },
  });

  return (
    <Page id="enrollment-start-page" nav>
      <EnrollmentStep current={1} max={2} />
      <header>
        <h1>{m.enrollment_start_title()}</h1>
        <p>{m.enrollment_start_subtitle()}</p>
      </header>
      <SizedBox height={ThemeSpacing.Xl5} />
      <ContainerWithIcon icon="globe">
        <header>
          <h5>{m.enrollment_start_external_title()}</h5>
          <p>{m.enrollment_start_external_subtitle()}</p>
        </header>
      </ContainerWithIcon>
      <Divider text={m.misc_or()} orientation="horizontal" />
      <ContainerWithIcon icon="file">
        <header>
          <h5>{m.enrollment_start_internal_title()}</h5>
          <p>{m.enrollment_start_internal_subtitle()}</p>
        </header>
        <form.AppForm>
          <form
            onSubmit={(e) => {
              e.preventDefault();
              e.stopPropagation();
              form.handleSubmit();
            }}
          >
            <form.AppField name="token">
              {(field) => <field.FormInput required label={m.form_label_token()} />}
            </form.AppField>
          </form>
        </form.AppForm>
      </ContainerWithIcon>
      <PageNavigation
        backText={m.controls_back()}
        onBack={() => {
          navigate({
            to: '/',
          });
        }}
        nextText={m.controls_continue()}
        onNext={() => {
          form.handleSubmit();
        }}
      />
    </Page>
  );
};
