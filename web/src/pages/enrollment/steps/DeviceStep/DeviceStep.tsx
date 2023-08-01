import './style.scss';

import { ConfigureDeviceCard } from './components/ConfigureDeviceCard/ConfigureDeviceCard';
import { QuickGuideCard } from './components/QuickGuideCard/QuickGuideCard';

export const DeviceStep = () => {
  return (
    <div id="enrollment-device-step">
      <ConfigureDeviceCard />
      <QuickGuideCard />
    </div>
  );
};
