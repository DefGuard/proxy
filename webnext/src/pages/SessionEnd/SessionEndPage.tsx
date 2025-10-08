import { m } from '../../paraglide/messages';
import { PageProcessEnd } from '../../shared/components/PageProcessEnd/PageProcessEnd';

export const SessionEndPage = () => {
  return (
    <PageProcessEnd
      link="/"
      icon="disabled"
      title={m.session_end_title()}
      subtitle={m.session_end_subtitle()}
      linkText={m.session_end_link()}
    />
  );
};
