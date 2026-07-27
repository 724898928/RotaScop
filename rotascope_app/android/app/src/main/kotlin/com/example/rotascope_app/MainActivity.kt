package com.example.rotascope_app

import android.graphics.ImageFormat
import android.graphics.PixelFormat
import android.media.Image
import android.media.ImageReader
import android.media.MediaCodec
import android.media.MediaCodecInfo
import android.media.MediaFormat
import android.os.Build
import android.view.Surface
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel
import java.nio.ByteBuffer

class MainActivity : FlutterActivity() {
    private val h264Channel = "rotascope/h264"
    private val quicChannel = "rotascope/quic"
    private var mediaCodec: MediaCodec? = null
    private var imageReader: ImageReader? = null
    private var codecSurface: Surface? = null
    private var frameWidth = 0
    private var frameHeight = 0
    private val info = MediaCodec.BufferInfo()
    private var latestImage: Image? = null
    private val imageLock = Any()

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)

        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, h264Channel)
            .setMethodCallHandler { call, result ->
                when (call.method) {
                    "init" -> {
                        val width = call.argument<Int>("width") ?: 1920
                        val height = call.argument<Int>("height") ?: 1080
                        result.success(initCodec(width, height))
                    }
                    "decode" -> {
                        val data = call.argument<ByteArray>("data")
                        if (data != null) {
                            feedDecoder(data)
                            val decoded = pollDecodedFrame()
                            result.success(decoded)
                        } else {
                            result.success(null)
                        }
                    }
                    "release" -> {
                        releaseCodec()
                        result.success(null)
                    }
                    else -> result.notImplemented()
                }
            }

        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, quicChannel)
            .setMethodCallHandler { call, result ->
                when (call.method) {
                    "connect" -> result.success(false)
                    "exchange" -> result.success(null)
                    "disconnect" -> result.success(null)
                    else -> result.notImplemented()
                }
            }
    }

    private fun initCodec(width: Int, height: Int): Boolean {
        return try {
            frameWidth = width
            frameHeight = height
            val format = MediaFormat.createVideoFormat(MediaFormat.MIMETYPE_VIDEO_AVC, width, height)
            format.setInteger(MediaFormat.KEY_FRAME_RATE, 60)
            format.setInteger(MediaFormat.KEY_I_FRAME_INTERVAL, 1)

            val reader = ImageReader.newInstance(width, height, ImageFormat.YUV_420_888, 3)
            imageReader = reader
            codecSurface = reader.surface

            val codec = MediaCodec.createDecoderByType(MediaFormat.MIMETYPE_VIDEO_AVC)
            codec.configure(format, reader.surface, null, 0)
            codec.start()
            mediaCodec = codec
            true
        } catch (e: Exception) {
            e.printStackTrace()
            false
        }
    }

    private fun feedDecoder(data: ByteArray) {
        val codec = mediaCodec ?: return
        try {
            val inputIndex = codec.dequeueInputBuffer(10000L)
            if (inputIndex < 0) return

            val inputBuffer = codec.getInputBuffer(inputIndex) ?: return
            inputBuffer.clear()
            inputBuffer.put(data)
            codec.queueInputBuffer(inputIndex, 0, data.size, System.nanoTime() / 1000, 0)

            val outputIndex = codec.dequeueOutputBuffer(info, 10000L)
            if (outputIndex >= 0) {
                synchronized(imageLock) {
                    latestImage = imageReader?.acquireLatestImage()
                }
                codec.releaseOutputBuffer(outputIndex, true)
            }
        } catch (_: Exception) {}
    }

    private fun pollDecodedFrame(): ByteArray? {
        synchronized(imageLock) {
            val image = latestImage ?: return null
            latestImage = null

            val planes = image.planes
            if (planes.size < 3) {
                image.close()
                return null
            }

            val yPlane = planes[0]
            val uPlane = planes[1]
            val vPlane = planes[2]

            val yBuf = yPlane.buffer
            val uBuf = uPlane.buffer
            val vBuf = vPlane.buffer

            val yRowStride = yPlane.rowStride
            val uRowStride = uPlane.rowStride
            val vRowStride = vPlane.rowStride

            val w = image.width
            val h = image.height

            val rgba = ByteArray(w * h * 4)
            val yRow = ByteArray(yRowStride)
            val uRow = ByteArray(uRowStride)
            val vRow = ByteArray(vRowStride)

            for (row in 0 until h) {
                yBuf.get(yRow, 0, minOf(yRowStride, yBuf.remaining()))
                uBuf.get(uRow, 0, minOf(uRowStride, uBuf.remaining()))
                vBuf.get(vRow, 0, minOf(vRowStride, vBuf.remaining()))

                for (col in 0 until w) {
                    val y = yRow[col].toInt() and 0xFF
                    val u = uRow[col / 2].toInt() and 0xFF
                    val v = vRow[col / 2].toInt() and 0xFF

                    val r = (y + 1.402 * (v - 128)).toInt().coerceIn(0, 255)
                    val g = (y - 0.344 * (u - 128) - 0.714 * (v - 128)).toInt().coerceIn(0, 255)
                    val b = (y + 1.772 * (u - 128)).toInt().coerceIn(0, 255)

                    val idx = (row * w + col) * 4
                    rgba[idx] = r.toByte()
                    rgba[idx + 1] = g.toByte()
                    rgba[idx + 2] = b.toByte()
                    rgba[idx + 3] = 0xFF.toByte()
                }

                if (row < h - 1) {
                    yBuf.position(minOf(yBuf.position() + yRowStride - w, yBuf.limit()))
                }
            }

            image.close()
            return rgba
        }
    }

    private fun releaseCodec() {
        try {
            mediaCodec?.stop()
            mediaCodec?.release()
        } catch (_: Exception) {}
        imageReader?.close()
        codecSurface?.release()
        mediaCodec = null
        imageReader = null
        codecSurface = null
    }

    override fun onDestroy() {
        releaseCodec()
        super.onDestroy()
    }
}
