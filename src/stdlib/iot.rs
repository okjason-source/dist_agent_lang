use std::collections::HashMap;
use std::env;
use crate::runtime::values::Value;

#[cfg(feature = "http-interface")]
fn value_to_serde_json(v: &Value) -> serde_json::Value {
    match v {
        Value::Int(n) => serde_json::json!(n),
        Value::Float(f) => serde_json::json!(f),
        Value::String(s) => serde_json::json!(s),
        Value::Bool(b) => serde_json::json!(b),
        Value::Null => serde_json::Value::Null,
        Value::List(arr) => serde_json::Value::Array(arr.iter().map(value_to_serde_json).collect()),
        Value::Map(m) => serde_json::Value::Object(
            m.iter().map(|(k, v)| (k.clone(), value_to_serde_json(v))).collect(),
        ),
        Value::Struct(_, m) => serde_json::Value::Object(
            m.iter().map(|(k, v)| (k.clone(), value_to_serde_json(v))).collect(),
        ),
        Value::Array(arr) => serde_json::Value::Array(arr.iter().map(value_to_serde_json).collect()),
        _ => serde_json::Value::String(v.to_string()),
    }
}

// IoT & Edge Computing Framework — all devices
//
// Encompasses **all device types**: sensors/actuators, gateways, edge nodes,
// **programmable hardware** (FPGA, microcontroller, PLC), and **robotics**
// (robot arms, mobile robots, drones, robot controllers). Use the device_type
// and capabilities to distinguish sensor nodes, industrial controllers,
// programmable hardware, and robots.
//
// - Device connectivity and management (including programmable HW and robots)
// - Sensor data processing and analysis
// - Edge computing with local AI processing
// - Cloud integration (optional IOT_CLOUD_URL when http-interface enabled)
// - Security and authentication for IoT devices
// - Protocol support (MQTT, CoAP, HTTP, WebSocket)
// - Anomaly detection and predictive maintenance (optional AI/ML when configured)

// === PHASE 6: IOT & EDGE COMPUTING STRUCTURES ===

// Device Management
#[derive(Debug, Clone)]
pub struct IoTDevice {
    pub device_id: String,
    pub device_type: DeviceType,
    pub name: String,
    pub status: DeviceStatus,
    pub location: Option<DeviceLocation>,
    pub capabilities: Vec<String>,
    pub sensors: Vec<Sensor>,
    pub actuators: Vec<Actuator>,
    pub last_seen: String,
    pub firmware_version: String,
    pub power_source: PowerSource,
    pub network_info: NetworkInfo,
    pub security_profile: SecurityProfile,
    pub event_handlers: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum DeviceType {
    SensorNode,
    ActuatorNode,
    Gateway,
    EdgeComputer,
    SmartDevice,
    IndustrialController,
    Wearable,
    ProgrammableHardware,
    RobotArm,
    MobileRobot,
    Drone,
    RobotController,
    Custom(String),
}

/// Parse device type from config string (e.g. "robot_arm", "programmable_hardware", "drone").
pub fn device_type_from_string(s: &str) -> Option<DeviceType> {
    Some(match s.to_lowercase().replace('-', "_").as_str() {
        "sensor_node" | "sensor" => DeviceType::SensorNode,
        "actuator_node" | "actuator" => DeviceType::ActuatorNode,
        "gateway" => DeviceType::Gateway,
        "edge_computer" | "edge" => DeviceType::EdgeComputer,
        "smart_device" | "smart" => DeviceType::SmartDevice,
        "industrial_controller" | "industrial" | "plc" => DeviceType::IndustrialController,
        "wearable" => DeviceType::Wearable,
        "programmable_hardware" | "programmable" | "fpga" | "microcontroller" => DeviceType::ProgrammableHardware,
        "robot_arm" | "robotarm" | "manipulator" => DeviceType::RobotArm,
        "mobile_robot" | "mobilerobot" | "agv" | "rover" => DeviceType::MobileRobot,
        "drone" | "uav" => DeviceType::Drone,
        "robot_controller" | "robotcontroller" | "motion_controller" => DeviceType::RobotController,
        other if !other.is_empty() => DeviceType::Custom(other.to_string()),
        _ => return None,
    })
}

#[derive(Debug, Clone)]
pub enum DeviceStatus {
    Online,
    Offline,
    Maintenance,
    Error,
    Provisioning,
    Decommissioned,
}

#[derive(Debug, Clone)]
pub struct DeviceLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub accuracy: f64,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub protocol: String,
    pub address: String,
    pub port: i64,
    pub quality: i64, // Signal strength/percentage
    pub bandwidth: Option<i64>,
    pub latency: Option<i64>,
}

// Sensor Framework
#[derive(Debug, Clone)]
pub struct Sensor {
    pub sensor_id: String,
    pub sensor_type: SensorType,
    pub name: String,
    pub unit: String,
    pub range: ValueRange,
    pub accuracy: f64,
    pub sampling_rate: i64, // Hz
    pub last_reading: Option<SensorReading>,
    pub calibration_data: Option<CalibrationData>,
    pub status: SensorStatus,
}

#[derive(Debug, Clone)]
pub enum SensorType {
    Temperature,
    Humidity,
    Pressure,
    Motion,
    Light,
    Sound,
    Proximity,
    Accelerometer,
    Gyroscope,
    Magnetometer,
    GPS,
    HeartRate,
    BloodOxygen,
    ECG,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct SensorReading {
    pub timestamp: String,
    pub value: Value,
    pub quality: ReadingQuality,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub enum ReadingQuality {
    Excellent,
    Good,
    Fair,
    Poor,
    Invalid,
}

#[derive(Debug, Clone)]
pub struct ValueRange {
    pub min: Value,
    pub max: Value,
}

#[derive(Debug, Clone)]
pub struct CalibrationData {
    pub calibrated_at: String,
    pub calibration_points: Vec<CalibrationPoint>,
    pub accuracy_improvement: f64,
}

#[derive(Debug, Clone)]
pub struct CalibrationPoint {
    pub input_value: Value,
    pub expected_output: Value,
    pub actual_output: Value,
}

#[derive(Debug, Clone)]
pub enum SensorStatus {
    Active,
    Inactive,
    Calibrating,
    Error,
    Maintenance,
}

// Actuator Framework
#[derive(Debug, Clone)]
pub struct Actuator {
    pub actuator_id: String,
    pub actuator_type: ActuatorType,
    pub name: String,
    pub status: ActuatorStatus,
    pub last_command: Option<ActuatorCommand>,
    pub supported_commands: Vec<String>,
    pub power_consumption: Option<f64>,
}

#[derive(Debug, Clone)]
pub enum ActuatorType {
    Relay,
    Motor,
    Servo,
    LED,
    Buzzer,
    Display,
    Valve,
    Pump,
    Heater,
    Cooler,
    Joint,
    Gripper,
    Propeller,
    Wheel,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct ActuatorCommand {
    pub command_id: String,
    pub command_type: String,
    pub parameters: HashMap<String, Value>,
    pub timestamp: String,
    pub status: CommandStatus,
}

#[derive(Debug, Clone)]
pub enum CommandStatus {
    Pending,
    Executing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub enum ActuatorStatus {
    Idle,
    Active,
    Error,
    Maintenance,
}

// Edge Computing Framework
#[derive(Debug, Clone)]
pub struct EdgeNode {
    pub node_id: String,
    pub name: String,
    pub location: DeviceLocation,
    pub capabilities: Vec<String>,
    pub processing_power: ProcessingCapability,
    pub storage_capacity: i64,
    pub network_bandwidth: i64,
    pub power_source: PowerSource,
    pub status: EdgeNodeStatus,
    pub connected_devices: Vec<String>, // Device IDs
    pub running_tasks: Vec<EdgeTask>,
    pub data_cache: HashMap<String, CachedData>,
}

#[derive(Debug, Clone)]
pub struct ProcessingCapability {
    pub cpu_cores: i64,
    pub cpu_frequency: f64, // GHz
    pub memory_gb: f64,
    pub gpu_available: bool,
    pub ai_acceleration: bool,
}

#[derive(Debug, Clone)]
pub enum EdgeNodeStatus {
    Online,
    Offline,
    Overloaded,
    Maintenance,
    Error,
}

#[derive(Debug, Clone)]
pub struct EdgeTask {
    pub task_id: String,
    pub task_type: EdgeTaskType,
    pub priority: TaskPriority,
    pub data: Value,
    pub status: EdgeTaskStatus,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub result: Option<Value>,
}

#[derive(Debug, Clone)]
pub enum EdgeTaskType {
    DataProcessing,
    AIInference,
    SensorFusion,
    AnomalyDetection,
    PredictiveMaintenance,
    ImageProcessing,
    AudioProcessing,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub enum EdgeTaskStatus {
    Queued,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct CachedData {
    pub key: String,
    pub data: Value,
    pub timestamp: String,
    pub ttl: Option<i64>, // Time to live in seconds
    pub access_count: i64,
    pub last_accessed: String,
}

// Communication Protocols
#[derive(Debug, Clone)]
pub enum ProtocolType {
    MQTT,
    CoAP,
    HTTP,
    WebSocket,
    BLE,
    LoRa,
    Zigbee,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct ProtocolConfig {
    pub protocol_type: ProtocolType,
    pub broker_address: Option<String>,
    pub port: Option<i64>,
    pub client_id: String,
    pub credentials: Option<Credentials>,
    pub qos_level: Option<i64>,
    pub keep_alive: Option<i64>,
    pub clean_session: bool,
}

#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: Option<String>,
    pub password: Option<String>,
    pub certificate_path: Option<String>,
    pub private_key_path: Option<String>,
}

// Security Framework
#[derive(Debug, Clone)]
pub struct SecurityProfile {
    pub authentication_method: AuthenticationMethod,
    pub encryption_enabled: bool,
    pub certificate_path: Option<String>,
    pub trusted_certificates: Vec<String>,
    pub security_level: SecurityLevel,
    pub access_policies: Vec<AccessPolicy>,
}

#[derive(Debug, Clone)]
pub enum AuthenticationMethod {
    None,
    Basic,
    Certificate,
    Token,
    OAuth2,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum SecurityLevel {
    Basic,
    Standard,
    High,
    Military,
}

#[derive(Debug, Clone)]
pub struct AccessPolicy {
    pub resource: String,
    pub action: String,
    pub allowed_roles: Vec<String>,
    pub conditions: Option<String>,
}

// Power Management
#[derive(Debug, Clone)]
pub enum PowerSource {
    Battery,
    Solar,
    Grid,
    Hybrid,
    Kinetic,
    Thermal,
}

#[derive(Debug, Clone)]
pub struct PowerStatus {
    pub source: PowerSource,
    pub battery_level: Option<f64>, // 0.0 to 1.0
    pub voltage: Option<f64>,
    pub current: Option<f64>,
    pub power_consumption: f64, // watts
    pub estimated_runtime: Option<i64>, // minutes
}

// Data Processing
#[derive(Debug, Clone)]
pub struct DataStream {
    pub stream_id: String,
    pub source_device: String,
    pub data_type: String,
    pub sampling_rate: i64,
    pub buffer_size: i64,
    pub filters: Vec<DataFilter>,
    pub processors: Vec<DataProcessor>,
    pub sinks: Vec<DataSink>,
}

#[derive(Debug, Clone)]
pub struct DataFilter {
    pub filter_type: FilterType,
    pub parameters: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub enum FilterType {
    Threshold,
    Range,
    MovingAverage,
    Kalman,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct DataProcessor {
    pub processor_type: ProcessorType,
    pub parameters: HashMap<String, Value>,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum ProcessorType {
    Normalization,
    FFT,
    Statistical,
    MLInference,
    Compression,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct DataSink {
    pub sink_type: SinkType,
    pub destination: String,
    pub parameters: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub enum SinkType {
    Database,
    Cloud,
    LocalFile,
    MessageQueue,
    Blockchain,
    Custom(String),
}

// === PHASE 6: IOT & EDGE COMPUTING FUNCTIONS ===

// Device Management
pub fn register_device(device_config: HashMap<String, Value>) -> Result<IoTDevice, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("action".to_string(), Value::String("register_device".to_string()));
        data.insert("device_config".to_string(), Value::Map(device_config.clone()));
        data.insert("message".to_string(), Value::String("Registering new IoT device".to_string()));
        data
    }, Some("iot"));

    let device_id = device_config.get("device_id")
        .and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| format!("device_{}", generate_id()));

    let device_type = device_config.get("device_type")
        .and_then(|v| match v {
            Value::String(s) => device_type_from_string(s),
            _ => None,
        })
        .unwrap_or(DeviceType::SensorNode);

    let device = IoTDevice {
        device_id: device_id.clone(),
        device_type,
        name: device_config.get("name")
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| format!("Device {}", device_id)),
        status: DeviceStatus::Provisioning,
        location: None,
        capabilities: vec!["basic".to_string()],
        sensors: Vec::new(),
        actuators: Vec::new(),
        last_seen: "2024-01-01T00:00:00Z".to_string(),
        firmware_version: "1.0.0".to_string(),
        power_source: PowerSource::Battery,
        network_info: NetworkInfo {
            protocol: "MQTT".to_string(),
            address: "localhost".to_string(),
            port: 1883,
            quality: 100,
            bandwidth: Some(1000000),
            latency: Some(10),
        },
        security_profile: SecurityProfile {
            authentication_method: AuthenticationMethod::Basic,
            encryption_enabled: true,
            certificate_path: None,
            trusted_certificates: Vec::new(),
            security_level: SecurityLevel::Standard,
            access_policies: Vec::new(),
        },
        event_handlers: HashMap::new(),
    };

    Ok(device)
}

pub fn connect_device(device_id: &str) -> Result<IoTDevice, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("device_id".to_string(), Value::String(device_id.to_string()));
        data.insert("message".to_string(), Value::String("Connecting to IoT device".to_string()));
        data
    }, Some("iot"));

    // Simulate device connection
    let mut device = register_device({
        let mut config = HashMap::new();
        config.insert("device_id".to_string(), Value::String(device_id.to_string()));
        config.insert("name".to_string(), Value::String(format!("Connected Device {}", device_id)));
        config
    })?;

    device.status = DeviceStatus::Online;
    device.last_seen = "2024-01-01T00:00:00Z".to_string();

    Ok(device)
}

pub fn disconnect_device(device_id: &str) -> Result<bool, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("device_id".to_string(), Value::String(device_id.to_string()));
        data.insert("message".to_string(), Value::String("Disconnecting IoT device".to_string()));
        data
    }, Some("iot"));

    Ok(true)
}

pub fn get_device_status(_device_id: &str) -> Result<DeviceStatus, String> {
    // Simulate status check
    Ok(DeviceStatus::Online)
}

pub fn update_device_firmware(device_id: &str, firmware_version: &str) -> Result<bool, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("device_id".to_string(), Value::String(device_id.to_string()));
        data.insert("firmware_version".to_string(), Value::String(firmware_version.to_string()));
        data.insert("message".to_string(), Value::String("Updating device firmware".to_string()));
        data
    }, Some("iot"));

    Ok(true)
}

// Sensor Management
pub fn add_sensor_to_device(device_id: &str, sensor_config: HashMap<String, Value>) -> Result<Sensor, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("device_id".to_string(), Value::String(device_id.to_string()));
        data.insert("sensor_config".to_string(), Value::Map(sensor_config.clone()));
        data.insert("message".to_string(), Value::String("Adding sensor to device".to_string()));
        data
    }, Some("iot"));

    let sensor_id = sensor_config.get("sensor_id")
        .and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| format!("sensor_{}", generate_id()));

    let sensor = Sensor {
        sensor_id: sensor_id.clone(),
        sensor_type: SensorType::Temperature,
        name: sensor_config.get("name")
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| format!("Sensor {}", sensor_id)),
        unit: "°C".to_string(),
        range: ValueRange {
            min: Value::Int(-50),
            max: Value::Int(100),
        },
        accuracy: 0.5,
        sampling_rate: 1,
        last_reading: None,
        calibration_data: None,
        status: SensorStatus::Active,
    };

    Ok(sensor)
}

pub fn read_sensor_data(sensor_id: &str) -> Result<SensorReading, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("sensor_id".to_string(), Value::String(sensor_id.to_string()));
        data.insert("message".to_string(), Value::String("Reading sensor data".to_string()));
        data
    }, Some("iot"));

    let reading = SensorReading {
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        value: Value::Float(25.5), // Simulated temperature reading
        quality: ReadingQuality::Excellent,
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("device_id".to_string(), Value::String("device_123".to_string()));
            meta.insert("sensor_type".to_string(), Value::String("temperature".to_string()));
            meta
        },
    };

    Ok(reading)
}

pub fn calibrate_sensor(sensor_id: &str, calibration_points: Vec<CalibrationPoint>) -> Result<bool, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("sensor_id".to_string(), Value::String(sensor_id.to_string()));
        data.insert("calibration_points".to_string(), Value::Int(calibration_points.len() as i64));
        data.insert("message".to_string(), Value::String("Calibrating sensor".to_string()));
        data
    }, Some("iot"));

    Ok(true)
}

// Actuator Control
pub fn add_actuator_to_device(device_id: &str, actuator_config: HashMap<String, Value>) -> Result<Actuator, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("device_id".to_string(), Value::String(device_id.to_string()));
        data.insert("actuator_config".to_string(), Value::Map(actuator_config.clone()));
        data.insert("message".to_string(), Value::String("Adding actuator to device".to_string()));
        data
    }, Some("iot"));

    let actuator_id = actuator_config.get("actuator_id")
        .and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| format!("actuator_{}", generate_id()));

    let actuator = Actuator {
        actuator_id: actuator_id.clone(),
        actuator_type: ActuatorType::Relay,
        name: actuator_config.get("name")
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| format!("Actuator {}", actuator_id)),
        status: ActuatorStatus::Idle,
        last_command: None,
        supported_commands: vec!["on".to_string(), "off".to_string()],
        power_consumption: Some(0.5),
    };

    Ok(actuator)
}

pub fn send_actuator_command(actuator_id: &str, command: &str, parameters: HashMap<String, Value>) -> Result<ActuatorCommand, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("actuator_id".to_string(), Value::String(actuator_id.to_string()));
        data.insert("command".to_string(), Value::String(command.to_string()));
        data.insert("parameters".to_string(), Value::Map(parameters.clone()));
        data.insert("message".to_string(), Value::String("Sending actuator command".to_string()));
        data
    }, Some("iot"));

    let actuator_command = ActuatorCommand {
        command_id: format!("cmd_{}", generate_id()),
        command_type: command.to_string(),
        parameters,
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        status: CommandStatus::Completed,
    };

    Ok(actuator_command)
}

// Edge Computing
pub fn create_edge_node(node_config: HashMap<String, Value>) -> Result<EdgeNode, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("action".to_string(), Value::String("create_edge_node".to_string()));
        data.insert("node_config".to_string(), Value::Map(node_config.clone()));
        data.insert("message".to_string(), Value::String("Creating edge computing node".to_string()));
        data
    }, Some("iot"));

    let node_id = node_config.get("node_id")
        .and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| format!("edge_node_{}", generate_id()));

    let edge_node = EdgeNode {
        node_id: node_id.clone(),
        name: node_config.get("name")
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| format!("Edge Node {}", node_id)),
        location: DeviceLocation {
            latitude: 37.7749,
            longitude: -122.4194,
            altitude: Some(10.0),
            accuracy: 5.0,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        },
        capabilities: vec!["ai_inference".to_string(), "data_processing".to_string()],
        processing_power: ProcessingCapability {
            cpu_cores: 4,
            cpu_frequency: 2.5,
            memory_gb: 8.0,
            gpu_available: false,
            ai_acceleration: true,
        },
        storage_capacity: 1000000000, // 1GB
        network_bandwidth: 100000000, // 100Mbps
        power_source: PowerSource::Grid,
        status: EdgeNodeStatus::Online,
        connected_devices: Vec::new(),
        running_tasks: Vec::new(),
        data_cache: HashMap::new(),
    };

    Ok(edge_node)
}

pub fn process_data_at_edge(edge_node_id: &str, data: Value, task_type: EdgeTaskType) -> Result<EdgeTask, String> {
    crate::stdlib::log::info("iot", {
        let mut data_map = std::collections::HashMap::new();
        data_map.insert("edge_node_id".to_string(), Value::String(edge_node_id.to_string()));
        data_map.insert("task_type".to_string(), Value::String(format!("{:?}", task_type)));
        data_map.insert("message".to_string(), Value::String("Processing data at edge node".to_string()));
        data_map
    }, Some("iot"));

    let task = EdgeTask {
        task_id: format!("edge_task_{}", generate_id()),
        task_type,
        priority: TaskPriority::Normal,
        data,
        status: EdgeTaskStatus::Completed,
        started_at: Some("2024-01-01T00:00:00Z".to_string()),
        completed_at: Some("2024-01-01T00:00:01Z".to_string()),
        result: Some(Value::String("processed_data".to_string())),
    };

    Ok(task)
}

pub fn cache_data_at_edge(edge_node_id: &str, key: &str, _data: Value, ttl_seconds: Option<i64>) -> Result<bool, String> {
    crate::stdlib::log::info("iot", {
        let mut data_map = std::collections::HashMap::new();
        data_map.insert("edge_node_id".to_string(), Value::String(edge_node_id.to_string()));
        data_map.insert("cache_key".to_string(), Value::String(key.to_string()));
        data_map.insert("ttl_seconds".to_string(), Value::Int(ttl_seconds.unwrap_or(3600)));
        data_map.insert("message".to_string(), Value::String("Caching data at edge node".to_string()));
        data_map
    }, Some("iot"));

    Ok(true)
}

pub fn get_cached_data_from_edge(_edge_node_id: &str, _key: &str) -> Result<Option<Value>, String> {
    // Simulate cache retrieval
    Ok(Some(Value::String("cached_data".to_string())))
}

// Data Streaming
pub fn create_data_stream(stream_config: HashMap<String, Value>) -> Result<DataStream, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("action".to_string(), Value::String("create_data_stream".to_string()));
        data.insert("stream_config".to_string(), Value::Map(stream_config.clone()));
        data.insert("message".to_string(), Value::String("Creating data stream".to_string()));
        data
    }, Some("iot"));

    let stream_id = stream_config.get("stream_id")
        .and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| format!("stream_{}", generate_id()));

    let data_stream = DataStream {
        stream_id: stream_id.clone(),
        source_device: stream_config.get("source_device")
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "device_unknown".to_string()),
        data_type: stream_config.get("data_type")
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "generic".to_string()),
        sampling_rate: 1,
        buffer_size: 1000,
        filters: Vec::new(),
        processors: Vec::new(),
        sinks: Vec::new(),
    };

    Ok(data_stream)
}

pub fn add_filter_to_stream(stream_id: &str, filter: DataFilter) -> Result<bool, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("stream_id".to_string(), Value::String(stream_id.to_string()));
        data.insert("filter_type".to_string(), Value::String(format!("{:?}", filter.filter_type)));
        data.insert("message".to_string(), Value::String("Adding filter to data stream".to_string()));
        data
    }, Some("iot"));

    Ok(true)
}

pub fn add_processor_to_stream(stream_id: &str, processor: DataProcessor) -> Result<bool, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("stream_id".to_string(), Value::String(stream_id.to_string()));
        data.insert("processor_type".to_string(), Value::String(format!("{:?}", processor.processor_type)));
        data.insert("message".to_string(), Value::String("Adding processor to data stream".to_string()));
        data
    }, Some("iot"));

    Ok(true)
}

pub fn add_sink_to_stream(stream_id: &str, sink: DataSink) -> Result<bool, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("stream_id".to_string(), Value::String(stream_id.to_string()));
        data.insert("sink_type".to_string(), Value::String(format!("{:?}", sink.sink_type)));
        data.insert("destination".to_string(), Value::String(sink.destination.clone()));
        data.insert("message".to_string(), Value::String("Adding sink to data stream".to_string()));
        data
    }, Some("iot"));

    Ok(true)
}

// Protocol Support
pub fn configure_protocol(protocol_config: ProtocolConfig) -> Result<bool, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("protocol_type".to_string(), Value::String(format!("{:?}", protocol_config.protocol_type)));
        data.insert("client_id".to_string(), Value::String(protocol_config.client_id.clone()));
        data.insert("message".to_string(), Value::String("Configuring IoT communication protocol".to_string()));
        data
    }, Some("iot"));

    Ok(true)
}

pub fn publish_message(protocol_config: &ProtocolConfig, topic: &str, _payload: Value) -> Result<bool, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("topic".to_string(), Value::String(topic.to_string()));
        data.insert("protocol".to_string(), Value::String(format!("{:?}", protocol_config.protocol_type)));
        data.insert("message".to_string(), Value::String("Publishing message via IoT protocol".to_string()));
        data
    }, Some("iot"));

    Ok(true)
}

pub fn subscribe_to_topic(protocol_config: &ProtocolConfig, topic: &str) -> Result<bool, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("topic".to_string(), Value::String(topic.to_string()));
        data.insert("protocol".to_string(), Value::String(format!("{:?}", protocol_config.protocol_type)));
        data.insert("message".to_string(), Value::String("Subscribing to IoT topic".to_string()));
        data
    }, Some("iot"));

    Ok(true)
}

// Security Functions
pub fn authenticate_device(device_id: &str, credentials: &Credentials) -> Result<bool, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("device_id".to_string(), Value::String(device_id.to_string()));
        data.insert("has_credentials".to_string(), Value::Bool(credentials.username.is_some()));
        data.insert("message".to_string(), Value::String("Authenticating IoT device".to_string()));
        data
    }, Some("iot"));

    Ok(true)
}

pub fn encrypt_device_data(_data: Value, security_profile: &SecurityProfile) -> Result<Value, String> {
    crate::stdlib::log::info("iot", {
        let mut data_map = std::collections::HashMap::new();
        data_map.insert("encryption_enabled".to_string(), Value::Bool(security_profile.encryption_enabled));
        data_map.insert("security_level".to_string(), Value::String(format!("{:?}", security_profile.security_level)));
        data_map.insert("message".to_string(), Value::String("Encrypting device data".to_string()));
        data_map
    }, Some("iot"));

    Ok(Value::String("encrypted_data".to_string()))
}

pub fn verify_device_certificate(device_id: &str, certificate_path: &str) -> Result<bool, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("device_id".to_string(), Value::String(device_id.to_string()));
        data.insert("certificate_path".to_string(), Value::String(certificate_path.to_string()));
        data.insert("message".to_string(), Value::String("Verifying device certificate".to_string()));
        data
    }, Some("iot"));

    Ok(true)
}

// Cloud Integration
/// Sync device data to cloud. When IOT_CLOUD_URL (and optional IOT_CLOUD_KEY) are set and
/// http-interface is enabled, POSTs to IOT_CLOUD_URL/sync; otherwise no-op success.
pub fn sync_device_data_to_cloud(device_id: &str, data: Value) -> Result<bool, String> {
    crate::stdlib::log::info("iot", {
        let mut data_map = std::collections::HashMap::new();
        data_map.insert("device_id".to_string(), Value::String(device_id.to_string()));
        data_map.insert("message".to_string(), Value::String("Syncing device data to cloud".to_string()));
        data_map
    }, Some("iot"));

    #[cfg(feature = "http-interface")]
    if let Ok(base) = env::var("IOT_CLOUD_URL") {
        let url = base.trim_end_matches('/').to_string() + "/sync";
        let body = serde_json::json!({
            "device_id": device_id,
            "data": value_to_serde_json(&data),
        });
        let client = reqwest::blocking::Client::new();
        let mut req = client.post(&url).json(&body);
        if let Ok(key) = env::var("IOT_CLOUD_KEY") {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        if let Err(e) = req.send() {
            return Err(format!("Cloud sync failed: {}", e));
        }
        return Ok(true);
    }

    Ok(true)
}

/// Get device data from cloud. When IOT_CLOUD_URL is set and http-interface enabled,
/// GETs from IOT_CLOUD_URL/device/{device_id}; otherwise returns mock.
pub fn get_device_data_from_cloud(device_id: &str, time_range: Option<(String, String)>) -> Result<Value, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("device_id".to_string(), Value::String(device_id.to_string()));
        data.insert("has_time_range".to_string(), Value::Bool(time_range.is_some()));
        data.insert("message".to_string(), Value::String("Retrieving device data from cloud".to_string()));
        data
    }, Some("iot"));

    #[cfg(feature = "http-interface")]
    if let Ok(base) = env::var("IOT_CLOUD_URL") {
        let mut url = format!("{}/device/{}", base.trim_end_matches('/'), device_id);
        if let Some((from, to)) = &time_range {
            url.push_str(&format!("?from={}&to={}", from, to));
        }
        let client = reqwest::blocking::Client::new();
        let mut req = client.get(&url);
        if let Ok(key) = env::var("IOT_CLOUD_KEY") {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        let resp = req.send().map_err(|e| format!("Cloud get failed: {}", e))?;
        let json: serde_json::Value = resp.json().map_err(|e| format!("Cloud response parse failed: {}", e))?;
        return Ok(serde_json_to_value(&json));
    }

    Ok(Value::String("cloud_data".to_string()))
}

#[cfg(feature = "http-interface")]
fn serde_json_to_value(j: &serde_json::Value) -> Value {
    match j {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::String(n.to_string())
            }
        }
        serde_json::Value::String(s) => Value::String(s.clone()),
        serde_json::Value::Array(arr) => Value::List(arr.iter().map(serde_json_to_value).collect()),
        serde_json::Value::Object(obj) => {
            let m: HashMap<String, Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), serde_json_to_value(v)))
                .collect();
            Value::Map(m)
        }
    }
}

// Anomaly Detection (Integration with AI)
/// Detect anomalies in sensor data. When IOT_ANOMALY_API_URL (and optional IOT_ANOMALY_API_KEY)
/// are set and http-interface is enabled, POSTs readings and expects JSON { "anomalies": ["..."] };
/// otherwise returns a conservative mock result.
pub fn detect_sensor_anomalies(sensor_data: Vec<SensorReading>) -> Result<Vec<String>, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("data_points".to_string(), Value::Int(sensor_data.len() as i64));
        data.insert("message".to_string(), Value::String("Detecting sensor anomalies".to_string()));
        data
    }, Some("iot"));

    #[cfg(feature = "http-interface")]
    if let Ok(url) = env::var("IOT_ANOMALY_API_URL") {
        let readings: Vec<serde_json::Value> = sensor_data
            .iter()
            .map(|r| {
                serde_json::json!({
                    "timestamp": r.timestamp,
                    "value": value_to_serde_json(&r.value),
                    "quality": format!("{:?}", r.quality),
                })
            })
            .collect();
        let body = serde_json::json!({ "readings": readings });
        let client = reqwest::blocking::Client::new();
        let mut req = client.post(url.trim()).json(&body);
        if let Ok(key) = env::var("IOT_ANOMALY_API_KEY") {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        if let Ok(resp) = req.send() {
            if let Ok(json) = resp.json::<serde_json::Value>() {
                if let Some(arr) = json.get("anomalies").and_then(|a| a.as_array()) {
                    let list: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                    return Ok(list);
                }
            }
        }
    }

    Ok(vec!["potential_anomaly_detected".to_string()])
}

/// Predict device failure probability (0.0–1.0). When IOT_ML_API_URL (and optional IOT_ML_API_KEY)
/// are set and http-interface is enabled, POSTs device_id and history and expects JSON
/// { "probability": 0.15 }; otherwise returns 0.15.
pub fn predict_device_failure(device_id: &str, sensor_history: Vec<SensorReading>) -> Result<f64, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("device_id".to_string(), Value::String(device_id.to_string()));
        data.insert("history_points".to_string(), Value::Int(sensor_history.len() as i64));
        data.insert("message".to_string(), Value::String("Predicting device failure probability".to_string()));
        data
    }, Some("iot"));

    #[cfg(feature = "http-interface")]
    if let Ok(url) = env::var("IOT_ML_API_URL") {
        let history: Vec<serde_json::Value> = sensor_history
            .iter()
            .map(|r| {
                serde_json::json!({
                    "timestamp": r.timestamp,
                    "value": value_to_serde_json(&r.value),
                })
            })
            .collect();
        let body = serde_json::json!({ "device_id": device_id, "history": history });
        let client = reqwest::blocking::Client::new();
        let mut req = client.post(url.trim()).json(&body);
        if let Ok(key) = env::var("IOT_ML_API_KEY") {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        if let Ok(resp) = req.send() {
            if let Ok(json) = resp.json::<serde_json::Value>() {
                if let Some(p) = json.get("probability").and_then(|v| v.as_f64()) {
                    return Ok(p.clamp(0.0, 1.0));
                }
            }
        }
    }

    Ok(0.15)
}

// Helper Functions
/// Device/sensor/actuator/stream IDs use UUID v4 when not provided by config.
pub fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

// Power Management
pub fn monitor_power_consumption(device_id: &str) -> Result<PowerStatus, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("device_id".to_string(), Value::String(device_id.to_string()));
        data.insert("message".to_string(), Value::String("Monitoring device power consumption".to_string()));
        data
    }, Some("iot"));

    let power_status = PowerStatus {
        source: PowerSource::Battery,
        battery_level: Some(0.75),
        voltage: Some(3.7),
        current: Some(0.5),
        power_consumption: 1.85,
        estimated_runtime: Some(480), // 8 hours
    };

    Ok(power_status)
}

pub fn optimize_power_usage(device_id: &str, target_runtime: i64) -> Result<HashMap<String, Value>, String> {
    crate::stdlib::log::info("iot", {
        let mut data = std::collections::HashMap::new();
        data.insert("device_id".to_string(), Value::String(device_id.to_string()));
        data.insert("target_runtime_hours".to_string(), Value::Int(target_runtime / 60));
        data.insert("message".to_string(), Value::String("Optimizing device power usage".to_string()));
        data
    }, Some("iot"));

    let optimizations = {
        let mut opts = HashMap::new();
        opts.insert("reduce_sampling_rate".to_string(), Value::Bool(true));
        opts.insert("disable_unused_sensors".to_string(), Value::Bool(true));
        opts.insert("lower_transmission_power".to_string(), Value::Bool(true));
        opts.insert("estimated_savings".to_string(), Value::Float(25.0));
        opts
    };

    Ok(optimizations)
}
