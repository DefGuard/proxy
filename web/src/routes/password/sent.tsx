import { createFileRoute } from '@tanstack/react-router';
import { m } from '../../paraglide/messages';
import { PageInfo } from '../../shared/components/PageInfo/PageInfo';

export const Route = createFileRoute('/password/sent')({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <PageInfo
      link="/"
      linkText={m.password_sent_link()}
      subtitle={m.password_sent_subTitle()}
      title={m.password_sent_title()}
    />
  );
}
