import { Folder, Camera, Heart, Settings, HelpCircle, LucideProps } from 'lucide-react';

const iconMap: Record<string, React.FC<LucideProps>> = {
    camera: Camera,
    heart: Heart,
    settings: Settings,
    folder: Folder,
};

const DynamicIcon = ({ name, ...props }: { name: string } & LucideProps) => {
    if (!name) return null;
    const Icon = iconMap[name];
    return <Icon {...props} />;
};