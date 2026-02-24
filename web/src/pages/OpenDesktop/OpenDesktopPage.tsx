import './style.scss';
import { useSearch } from '@tanstack/react-router';
import { m } from '../../paraglide/messages';
import { PageProcessEnd } from '../../shared/components/PageProcessEnd/PageProcessEnd';
import laptopImage from './assets/laptop.png';

export const OpenDesktopPage = () => {
  const { token } = useSearch({
    from: '/open-desktop',
  });

  const deepLinkUrl = new URL('defguard://addinstance');
  deepLinkUrl.searchParams.set('token', token);
  if (typeof window !== 'undefined') {
    deepLinkUrl.searchParams.set('url', window.location.origin);
  }

  return (
    <PageProcessEnd
      imageSrc={laptopImage}
      title={m.open_desktop_title()}
      subtitle={m.open_desktop_description()}
      link={deepLinkUrl.toString()}
      linkText={m.open_desktop_button()}
    />
  );
};
