import { m } from '../../paraglide/messages';
import { PageInfo } from '../../shared/components/PageInfo/PageInfo';

export const SessionEndPage = () => {
  return (
    <PageInfo
      link="/"
      icon="disabled"
      title={m.session_end_title()}
      subtitle={m.session_end_subtitle()}
      linkText={m.session_end_link()}
    />
  );
};
