use crate::{
    error::RuleEngineError,
    node::RuleNode,
    nodes::{
        ai::AiNode,
        debug::DebugNode,
        math::{CalculatedFieldNode, GetValueNode, RoundNode, StatisticsNode},
        profile::DeviceProfileRuleNode,
        transaction::{CalculatedFieldsSyncNode, EntityStateSyncNode},
        action::{
            AssignToCustomerNode, AwsLambdaNode, AwsSnsNode, AwsSqsNode,
            ClearAlarmNode, CreateAlarmNode, CreateRelationNode, DeleteRelationNode,
            GcpPubSubNode, KafkaNode, LogNode, MqttPublishNode, MsgCountNode,
            MsgDeduplicationNode, MsgGeneratorNode, MsgPushToEdgeNode,
            RabbitMqNode, RestApiCallNode, SaveAttributesNode,
            SaveTelemetryNode, SendEmailNode, SendNotificationNode, SendRpcReplyNode,
            SendRpcRequestNode, SendSmsNode, UnassignFromCustomerNode,
        },
        enrichment::{
            AggregateLatestNode, CalculateDeltaNode,
            CustomerAttributesNode, DeviceProfileNode, FetchDeviceCredentialsNode,
            GetAlarmDetailsNode, GetCustomerDetailsNode, GetDeviceStateNode,
            GetEntityDetailsNode, GetEntityFieldsNode, GetGeoDataNode,
            GetParentSummaryNode, GetTelemetryNode, GetTenantDetailsNode,
            GetUserAttributesNode, OriginatorAttributesNode, OriginatorFieldsNode,
            RelatedAttributesNode, TenantAttributesNode,
        },
        filter::{
            CheckAlarmStatusNode, CheckGeofenceNode, CheckMessageNode, CheckRelationNode,
            DeviceStateSwitchNode, GeofenceUpdateNode, GpsGeofencingActionNode,
            GpsGeofencingFilterNode, MsgTypeFilter, OriginatorTypeFilterNode,
            OriginatorTypeSwitchNode, ScriptFilter, ThresholdFilterNode,
        },
        flow::{
            AckNode, AssetTypeSwitchNode, CheckpointNode, DeviceTypeSwitchNode,
            MsgDelayNode, MsgTypeSwitchNode, RuleChainInputNode,
            RuleChainOutputNode, SynchronizationNode,
        },
        transform::{
            AssignAttributeNode, CalculateDistanceNode, ChangeOriginatorNode,
            CopyKeysNode, DeleteKeysNode, FormatTelemetryNode, MathNode, JsonPathNode,
            ParseMsgNode, RenameKeysNode, SplitArrayMsgNode, StringToJsonNode,
            ToEmailNode, TransformMsgNode,
        },
    },
};

/// Factory for creating RuleNode instances from type string + JSON config.
/// Node type strings match ThingsBoard Java class names for compatibility.
pub struct NodeRegistry;

impl NodeRegistry {
    pub fn create(
        &self,
        node_type: &str,
        config: &serde_json::Value,
    ) -> Result<Box<dyn RuleNode>, RuleEngineError> {
        match node_type {
            // ── Filter nodes ────────────────────────────────────────────────
            "MsgTypeFilter"
            | "TbMsgTypeFilterNode"    => Ok(Box::new(MsgTypeFilter::new(config)?)),

            "ScriptFilter"
            | "TbJsFilterNode"
            | "TbJsSwitchNode"         => Ok(Box::new(ScriptFilter::new(config)?)),

            "CheckMessage"
            | "TbCheckMessageNode"     => Ok(Box::new(CheckMessageNode::new(config)?)),

            "CheckRelation"
            | "TbCheckRelationNode"    => Ok(Box::new(CheckRelationNode::new(config)?)),

            "OriginatorTypeFilter"
            | "TbOriginatorTypeFilterNode"
                                       => Ok(Box::new(OriginatorTypeFilterNode::new(config)?)),

            "OriginatorTypeSwitch"
            | "TbOriginatorTypeSwitchNode"
                                       => Ok(Box::new(OriginatorTypeSwitchNode::new(config)?)),

            "ThresholdFilter"
            | "TbThresholdFilterNode"  => Ok(Box::new(ThresholdFilterNode::new(config)?)),

            // ── S2: New filter nodes ────────────────────────────────────────
            "CheckAlarmStatus"
            | "TbCheckAlarmStatusNode" => Ok(Box::new(CheckAlarmStatusNode::new(config)?)),

            "DeviceStateSwitch"
            | "TbDeviceStateSwitchNode"
                                       => Ok(Box::new(DeviceStateSwitchNode::new(config)?)),

            // ── S5: Geofence filter ─────────────────────────────────────────
            "CheckGeofence"
            | "TbCheckGeofenceNode"    => Ok(Box::new(CheckGeofenceNode::new(config)?)),

            "GeofenceUpdate"
            | "TbGeofenceUpdateNode"   => Ok(Box::new(GeofenceUpdateNode::new(config)?)),

            // ── S4: Profile state machine ───────────────────────────────────
            "DeviceProfile"
            | "TbDeviceProfileNode"    => Ok(Box::new(DeviceProfileRuleNode::new(config)?)),

            // ── Phase 25: Geo nodes ─────────────────────────────────────────
            "GpsGeofencingFilter"
            | "TbGpsGeofencingFilterNode"
                                       => Ok(Box::new(GpsGeofencingFilterNode::new(config)?)),

            "GpsGeofencingAction"
            | "TbGpsGeofencingActionNode"
                                       => Ok(Box::new(GpsGeofencingActionNode::new(config)?)),

            // ── Enrichment nodes ─────────────────────────────────────────────
            "OriginatorAttributes"
            | "TbGetOriginatorAttributesNode"
                                       => Ok(Box::new(OriginatorAttributesNode::new(config)?)),

            "OriginatorFields"
            | "TbGetOriginatorFieldsNode"
                                       => Ok(Box::new(OriginatorFieldsNode::new(config)?)),

            "CustomerAttributes"
            | "TbGetCustomerAttributesNode"
                                       => Ok(Box::new(CustomerAttributesNode::new(config)?)),

            "TenantAttributes"
            | "TbGetTenantAttributesNode"
                                       => Ok(Box::new(TenantAttributesNode::new(config)?)),

            "TbGetDeviceProfileNode"   => Ok(Box::new(DeviceProfileNode::new(config)?)),

            "RelatedAttributes"
            | "TbGetRelatedAttributesNode"
                                       => Ok(Box::new(RelatedAttributesNode::new(config)?)),

            "GetTelemetry"
            | "TbGetTelemetryNode"     => Ok(Box::new(GetTelemetryNode::new(config)?)),

            "GetCustomerDetails"
            | "TbGetCustomerDetailsNode"
                                       => Ok(Box::new(GetCustomerDetailsNode::new(config)?)),

            "GetTenantDetails"
            | "TbGetTenantDetailsNode" => Ok(Box::new(GetTenantDetailsNode::new(config)?)),

            "FetchDeviceCredentials"
            | "TbFetchDeviceCredentialsNode"
                                       => Ok(Box::new(FetchDeviceCredentialsNode::new(config)?)),

            // ── S5: Geo enrichment ───────────────────────────────────────────
            "GetGeoData"
            | "TbGetGeoDataNode"       => Ok(Box::new(GetGeoDataNode::new(config)?)),

            // ── S3: New enrichment nodes ──────────────────────────────────────
            "GetEntityDetails"
            | "TbGetEntityDetailsNode" => Ok(Box::new(GetEntityDetailsNode::new(config)?)),

            "GetAlarmDetails"
            | "TbGetAlarmDetailsNode"  => Ok(Box::new(GetAlarmDetailsNode::new(config)?)),

            "GetDeviceState"
            | "TbGetDeviceStateNode"   => Ok(Box::new(GetDeviceStateNode::new(config)?)),

            "CalculateDelta"
            | "TbCalculateDeltaNode"   => Ok(Box::new(CalculateDeltaNode::new(config)?)),

            "AggregateLatest"
            | "TbAggregateLatestNode"  => Ok(Box::new(AggregateLatestNode::new(config)?)),

            // ── S3: New enrichment nodes ──────────────────────────────────────

            "GetEntityFields"
            | "TbGetEntityFieldsNode"  => Ok(Box::new(GetEntityFieldsNode::new(config)?)),

            "GetParentSummary"
            | "TbGetParentSummaryNode" => Ok(Box::new(GetParentSummaryNode::new(config)?)),

            "GetUserAttributes"
            | "TbGetUserAttributesNode"
                                       => Ok(Box::new(GetUserAttributesNode::new(config)?)),

            // ── Transform nodes ──────────────────────────────────────────────
            "TransformMsg"
            | "TbTransformMsgNode"     => Ok(Box::new(TransformMsgNode::new(config)?)),

            "ChangeOriginator"
            | "TbChangeOriginatorNode" => Ok(Box::new(ChangeOriginatorNode::new(config)?)),

            "CopyKeys"
            | "TbCopyKeysNode"         => Ok(Box::new(CopyKeysNode::new(config)?)),

            "DeleteKeys"
            | "TbDeleteKeysNode"       => Ok(Box::new(DeleteKeysNode::new(config)?)),

            "RenameKeys"
            | "TbRenameKeysNode"       => Ok(Box::new(RenameKeysNode::new(config)?)),

            "ToEmail"
            | "TbMsgToEmailNode"       => Ok(Box::new(ToEmailNode::new(config)?)),

            "MathNode"
            | "TbMathNode"             => Ok(Box::new(MathNode::new(config)?)),

            "JsonPath"
            | "TbJsonPathNode"         => Ok(Box::new(JsonPathNode::new(config)?)),

            "SplitArrayMsg"
            | "TbSplitArrayMsgNode"    => Ok(Box::new(SplitArrayMsgNode::new(config)?)),

            // ── S5: Geo transform ────────────────────────────────────────────
            "CalculateDistance"
            | "TbCalculateDistanceNode"
                                       => Ok(Box::new(CalculateDistanceNode::new(config)?)),

            // ── S5: Math nodes ───────────────────────────────────────────────
            "GetValue"
            | "TbGetValueNode"         => Ok(Box::new(GetValueNode::new(config)?)),

            "Statistics"
            | "TbStatisticsNode"       => Ok(Box::new(StatisticsNode::new(config)?)),

            "CalculatedField"
            | "TbCalculatedFieldNode"
            | "TbRuleEngineCalculatedFieldNode"
                                       => Ok(Box::new(CalculatedFieldNode::new(config)?)),

            "Round"
            | "TbRoundNode"            => Ok(Box::new(RoundNode::new(config)?)),

            // ── S2: New transform nodes ──────────────────────────────────────
            "ParseMsg"
            | "TbParseMsgNode"         => Ok(Box::new(ParseMsgNode::new(config)?)),

            "StringToJson"
            | "TbStringToJsonNode"     => Ok(Box::new(StringToJsonNode::new(config)?)),

            "FormatTelemetry"
            | "TbFormatTelemetryNode"
            | "TbConvertTelemetryNode" => Ok(Box::new(FormatTelemetryNode::new(config)?)),

            "AssignAttribute"
            | "TbAssignAttributeNode"  => Ok(Box::new(AssignAttributeNode::new(config)?)),

            // ── Action nodes ─────────────────────────────────────────────────
            "SaveTelemetry"
            | "TbSendTelemetryNode"    => Ok(Box::new(SaveTelemetryNode::new(config)?)),

            "SaveAttributes"
            | "TbSendAttributesNode"   => Ok(Box::new(SaveAttributesNode::new(config)?)),

            "CreateAlarm"
            | "TbCreateAlarmNode"      => Ok(Box::new(CreateAlarmNode::new(config)?)),

            "ClearAlarm"
            | "TbClearAlarmNode"       => Ok(Box::new(ClearAlarmNode::new(config)?)),

            "RestApiCall"
            | "TbRestApiCallNode"      => Ok(Box::new(RestApiCallNode::new(config)?)),

            "MqttPublish"
            | "TbMsgPushToExternalMqtt"
                                       => Ok(Box::new(MqttPublishNode::new(config)?)),

            "Log"
            | "TbLogNode"              => Ok(Box::new(LogNode::new(config)?)),

            "SendNotification"
            | "TbSendNotificationNode" => Ok(Box::new(SendNotificationNode::new(config)?)),

            "CreateRelation"
            | "TbCreateRelationNode"   => Ok(Box::new(CreateRelationNode::new(config)?)),

            "DeleteRelation"
            | "TbDeleteRelationNode"   => Ok(Box::new(DeleteRelationNode::new(config)?)),

            // ── Phase 25: Device RPC ────────────────────────────────────────
            "SendRpcRequest"
            | "TbSendRPCRequestNode"   => Ok(Box::new(SendRpcRequestNode::new(config)?)),

            "SendRpcReply"
            | "TbSendRPCReplyNode"     => Ok(Box::new(SendRpcReplyNode::new(config)?)),

            // ── Phase 25: Messaging ─────────────────────────────────────────
            "SendEmail"
            | "TbSendEmailNode"        => Ok(Box::new(SendEmailNode::new(config)?)),

            "SendSms"
            | "TbSendSmsNode"          => Ok(Box::new(SendSmsNode::new(config)?)),

            "KafkaNode"
            | "TbKafkaNode"            => Ok(Box::new(KafkaNode::new(config)?)),

            "RabbitMqNode"
            | "TbRabbitMqNode"         => Ok(Box::new(RabbitMqNode::new(config)?)),

            // ── Phase 25: Cloud Integration ─────────────────────────────────
            "AwsLambda"
            | "TbAwsLambdaNode"        => Ok(Box::new(AwsLambdaNode::new(config)?)),

            "AwsSns"
            | "TbSnsNode"              => Ok(Box::new(AwsSnsNode::new(config)?)),

            "AwsSqs"
            | "TbSqsNode"              => Ok(Box::new(AwsSqsNode::new(config)?)),

            "GcpPubSub"
            | "TbPubSubNode"           => Ok(Box::new(GcpPubSubNode::new(config)?)),

            // ── Phase 25: Assignment & Stats ────────────────────────────────
            "AssignToCustomer"
            | "TbAssignToCustomerNode" => Ok(Box::new(AssignToCustomerNode::new(config)?)),

            "UnassignFromCustomer"
            | "TbUnassignFromCustomerNode"
                                       => Ok(Box::new(UnassignFromCustomerNode::new(config)?)),

            "MsgDeduplication"
            | "TbMsgDeduplicationNode" => Ok(Box::new(MsgDeduplicationNode::new(config)?)),

            "MsgCount"
            | "TbMsgCountNode"         => Ok(Box::new(MsgCountNode::new(config)?)),

            "MsgGenerator"
            | "TbMsgGeneratorNode"     => Ok(Box::new(MsgGeneratorNode::new(config)?)),

            "MsgPushToEdge"
            | "TbMsgPushToEdgeNode"    => Ok(Box::new(MsgPushToEdgeNode::new(config)?)),

            // ── Flow nodes ───────────────────────────────────────────────────
            "RuleChainInput"
            | "TbRuleChainInputNode"   => Ok(Box::new(RuleChainInputNode::new(config)?)),

            "MsgTypeSwitch"
            | "TbMsgTypeSwitchNode"    => Ok(Box::new(MsgTypeSwitchNode::new(config)?)),

            "Checkpoint"
            | "TbCheckpointNode"       => Ok(Box::new(CheckpointNode::new(config)?)),

            "DeviceTypeSwitch"
            | "TbDeviceTypeSwitchNode" => Ok(Box::new(DeviceTypeSwitchNode::new(config)?)),

            "AssetTypeSwitch"
            | "TbAssetTypeSwitchNode"  => Ok(Box::new(AssetTypeSwitchNode::new(config)?)),

            "Ack"
            | "TbAckNode"              => Ok(Box::new(AckNode::new(config)?)),

            "MsgDelay"
            | "TbMsgDelayNode"         => Ok(Box::new(MsgDelayNode::new(config)?)),

            "RuleChainOutput"
            | "TbRuleChainOutputNode"  => Ok(Box::new(RuleChainOutputNode::new(config)?)),

            "Synchronization"
            | "TbSynchronizationBeginNode"
            | "TbSynchronizationEndNode"
                                       => Ok(Box::new(SynchronizationNode::new(config)?)),

            // ── S11: Debug node ───────────────────────────────────────────────
            "DebugNode"
            | "TbDebugNode"            => Ok(Box::new(DebugNode::new(config)?)),

            // ── S6: AI nodes ─────────────────────────────────────────────────
            "AiNode"
            | "TbAiNode"               => Ok(Box::new(AiNode::new(config)?)),

            // ── S6: Transaction nodes ─────────────────────────────────────────
            "CalculatedFieldsSync"
            | "TbCalculatedFieldsSyncNode"
                                       => Ok(Box::new(CalculatedFieldsSyncNode::new(config)?)),

            "EntityStateSync"
            | "TbEntityStateSyncNode"  => Ok(Box::new(EntityStateSyncNode::new(config)?)),

            unknown => Err(RuleEngineError::Config(
                format!("Unknown node type: {}", unknown),
            )),
        }
    }
}
