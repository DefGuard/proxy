import { createFileRoute } from '@tanstack/react-router';
import { m } from '../../paraglide/messages';
import { PageInfo } from '../../shared/components/PageInfo/PageInfo';

const RouteComponent = () => {
  return (
    <PageInfo
      title={m.password_end_title()}
      subtitle={m.password_end_subtitle()}
      link="/"
      linkText={m.password_end_link()}
    />
  );
};

export const Route = createFileRoute('/password/finish')({
  component: RouteComponent,
});
