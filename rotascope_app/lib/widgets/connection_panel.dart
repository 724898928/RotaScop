import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../services/connection_service.dart';

class ConnectionPanel extends StatefulWidget {
  const ConnectionPanel({super.key});

  @override
  State<ConnectionPanel> createState() => _ConnectionPanelState();
}

class _ConnectionPanelState extends State<ConnectionPanel> {
  final _formKey = GlobalKey<FormState>();
  final _addressController = TextEditingController();

  @override
  void initState() {
    super.initState();
    final service = context.read<ConnectionService>();
    _addressController.text = service.serverAddress;
  }

  @override
  Widget build(BuildContext context) {
    final service = context.watch<ConnectionService>();
    final isBusy = service.status == ConnectionStatus.connecting;
    final isError = service.status == ConnectionStatus.error;

    return ColoredBox(
      color: const Color(0xFF05070A),
      child: SafeArea(
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 620),
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Form(
                key: _formKey,
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    Row(
                      children: [
                        Container(
                          width: 54,
                          height: 54,
                          decoration: BoxDecoration(
                            color: const Color(0xFF123A4A),
                            borderRadius: BorderRadius.circular(8),
                            border: Border.all(color: const Color(0xFF32D2C6)),
                          ),
                          child: const Icon(
                            Icons.screenshot_monitor,
                            color: Color(0xFF9FF5EE),
                            size: 30,
                          ),
                        ),
                        const SizedBox(width: 16),
                        const Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                'RotaScope USB Display',
                                style: TextStyle(
                                  color: Colors.white,
                                  fontSize: 22,
                                  fontWeight: FontWeight.w700,
                                ),
                              ),
                              SizedBox(height: 4),
                              Text(
                                '等待电脑端显示流',
                                style: TextStyle(
                                  color: Colors.white60,
                                  fontSize: 14,
                                ),
                              ),
                            ],
                          ),
                        ),
                        _StatusPill(status: service.status),
                      ],
                    ),
                    const SizedBox(height: 28),
                    TextFormField(
                      controller: _addressController,
                      autocorrect: false,
                      keyboardType: TextInputType.url,
                      style: const TextStyle(color: Colors.white),
                      decoration: InputDecoration(
                        labelText: '连接地址',
                        hintText: ConnectionService.defaultUsbAddress,
                        prefixIcon: const Icon(Icons.usb),
                        suffixIcon: IconButton(
                          tooltip: '使用 USB 默认地址',
                          onPressed: () {
                            _addressController.text =
                                ConnectionService.defaultUsbAddress;
                            service.useUsbDefaults();
                          },
                          icon: const Icon(Icons.settings_input_component),
                        ),
                        filled: true,
                        fillColor: Colors.white.withOpacity(0.07),
                        border: OutlineInputBorder(
                          borderRadius: BorderRadius.circular(8),
                        ),
                      ),
                      validator: (value) {
                        final text = value?.trim() ?? '';
                        if (text.isEmpty) return '请输入连接地址';
                        final uri = text.startsWith('ws://') ||
                                text.startsWith('wss://')
                            ? Uri.tryParse(text)
                            : Uri.tryParse('ws://$text');
                        if (uri == null || uri.host.isEmpty || !uri.hasPort) {
                          return '格式示例：127.0.0.1:8083/ws';
                        }
                        return null;
                      },
                      onChanged: service.updateServerAddress,
                    ),
                    const SizedBox(height: 18),
                    Row(
                      children: [
                        Expanded(
                          child: FilledButton.icon(
                            onPressed: isBusy
                                ? null
                                : () => _connectToServer(service),
                            icon: isBusy
                                ? const SizedBox.square(
                                    dimension: 18,
                                    child: CircularProgressIndicator(
                                      strokeWidth: 2,
                                    ),
                                  )
                                : const Icon(Icons.link),
                            label: Text(isBusy ? '连接中' : '连接'),
                          ),
                        ),
                        const SizedBox(width: 12),
                        IconButton.filledTonal(
                          tooltip: '停止重连',
                          onPressed: () => service.disconnect(),
                          icon: const Icon(Icons.link_off),
                        ),
                      ],
                    ),
                    if (isError || service.lastError != null) ...[
                      const SizedBox(height: 18),
                      Text(
                        service.lastError ?? '连接失败',
                        style: const TextStyle(
                          color: Color(0xFFFFB4AB),
                          fontSize: 13,
                        ),
                      ),
                    ],
                    if (service.reconnectAttempts > 0) ...[
                      const SizedBox(height: 8),
                      Text(
                        '自动重连次数：${service.reconnectAttempts}',
                        style: const TextStyle(
                          color: Colors.white54,
                          fontSize: 12,
                        ),
                      ),
                    ],
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }

  Future<void> _connectToServer(ConnectionService service) async {
    if (!_formKey.currentState!.validate()) return;

    service.updateServerAddress(_addressController.text);
    try {
      await service.connect(autoReconnect: true);
    } catch (_) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('电脑端暂未响应')),
      );
    }
  }

  @override
  void dispose() {
    _addressController.dispose();
    super.dispose();
  }
}

class _StatusPill extends StatelessWidget {
  const _StatusPill({required this.status});

  final ConnectionStatus status;

  @override
  Widget build(BuildContext context) {
    final (label, color, icon) = switch (status) {
      ConnectionStatus.connected => (
          '已连接',
          const Color(0xFF7DFFB2),
          Icons.check_circle,
        ),
      ConnectionStatus.connecting => (
          '连接中',
          const Color(0xFFFFD166),
          Icons.sync,
        ),
      ConnectionStatus.error => (
          '异常',
          const Color(0xFFFFB4AB),
          Icons.error,
        ),
      ConnectionStatus.disconnected => (
          '待连接',
          Colors.white60,
          Icons.radio_button_unchecked,
        ),
    };

    return DecoratedBox(
      decoration: BoxDecoration(
        color: color.withOpacity(0.12),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withOpacity(0.6)),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 15, color: color),
            const SizedBox(width: 6),
            Text(
              label,
              style: TextStyle(
                color: color,
                fontSize: 12,
                fontWeight: FontWeight.w600,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
