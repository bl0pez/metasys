export interface UmaPoint {
  id: string;
  name: string;
  reference: string;
  control_type: 'toggle' | 'analog' | 'temperature';
}

export interface Uma {
  id: string;
  name: string;
  points: UmaPoint[];
}

export interface QueuedCommand {
  id: string;
  uma_name: string;
  point_name: string;
  reference: string;
  action: string;
  retries: number;
  status: 'PENDING' | 'PROCESSING' | 'RETRYING' | 'COMPLETED' | 'FAILED';
}
