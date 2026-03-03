import React from 'react';
import { LaptopMinimalCheck, CircleAlert, ShieldAlert } from 'lucide-react';

const SidebarNotifications: React.FC = () => {
    return <div className="flex flex-row gap-2 items-center mr-2">
        <LaptopMinimalCheck className="text-green-500 size-5 mr-2" />
        <CircleAlert className="text-yellow-500 size-5" />
        {/* <ShieldAlert className="text-red-500 w-5 h-5" /> */}
    </div>
        ;
};

export default SidebarNotifications;