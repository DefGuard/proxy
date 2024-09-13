import './style.scss';

import { Divider } from '../../shared/components/layout/Divider/Divider';
import { DividerDirection } from '../../shared/components/layout/Divider/types';
import SvgDefguardLogoText from '../../shared/components/svg/DefguardLogoText';
import SvgTeoniteLogo from '../../shared/components/svg/TeoniteLogo';

export const LogoContainer = () => {
  return (
    <div className="logo-container">
      <SvgDefguardLogoText className="defguard" />
      <Divider direction={DividerDirection.VERTICAL} />
      <a href="https://teonite.com" target="_blank">
        <SvgTeoniteLogo className="teonite" />
      </a>
    </div>
  );
};
