import { RotateCw, Info } from 'lucide-react';
import { Button } from '../ui/button';

export function DemoBanner() {
    const isDemo = import.meta.env.VITE_USE_MOCK_API === 'true';

    if (!isDemo) return null;

    const handleReset = () => {
        if (confirm('Are you sure? This will reset all demo data (users, devices) to default.')) {
            localStorage.removeItem('wol_demo_users');
            localStorage.removeItem('wol_demo_devices');
            localStorage.removeItem('demo_access_token');
            window.location.reload();
        }
    };

    return (
        <div className="w-full bg-blue-600 text-white p-3 z-50 shadow-lg shrink-0">
            <div className="max-w-7xl mx-auto flex items-center justify-between gap-4">
                <div className="flex items-center gap-2">
                    <Info className="h-5 w-5 shrink-0" />
                    <span className="text-sm font-medium">
                        Demo Mode Active: No data is saved to a persistent backend. Changes are stored locally in your browser.
                    </span>
                </div>
                <Button 
                    variant="secondary" 
                    size="sm" 
                    className="whitespace-nowrap gap-2 bg-white text-blue-600 hover:bg-blue-50 border-0"
                    onClick={handleReset}
                >
                    <RotateCw className="h-4 w-4" />
                    Reset Demo
                </Button>
            </div>
        </div>
    );
}
