import { createFileRoute } from '@tanstack/react-router';
import { m } from '../paraglide/messages';
import { PageInfo } from '../shared/components/PageInfo/PageInfo';

export const Route = createFileRoute('/link-invalid')({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <PageInfo
      link="/"
      icon="disabled"
      title={m.link_invalid_title()}
      subtitle={m.link_invalid_subtitle()}
      linkText={m.session_end_link()}
    />
  );
}
