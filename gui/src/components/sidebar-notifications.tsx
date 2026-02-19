import React from 'react';
import { LaptopMinimalCheck, CircleAlert, ShieldAlert } from 'lucide-react';

const SidebarNotifications: React.FC = () => {
    return <div className="mb-2 flex flex-row gap-2 justify-end">
        <LaptopMinimalCheck className="text-green-500 w-6 h-6" />
        <CircleAlert className="text-yellow-500 w-6 h-6" />
        <ShieldAlert className="text-red-500 w-6 h-6" />
    </div>
        ;
};

export default SidebarNotifications;