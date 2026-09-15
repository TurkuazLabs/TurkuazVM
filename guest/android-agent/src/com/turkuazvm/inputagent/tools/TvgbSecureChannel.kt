// # 📄 Dosya Yolu: /turkuazvm/guest/android-agent/src/com/turkuazvm/inputagent/tools/TvgbSecureChannel.kt
// # 📌 Amac: Android Guest Agent icin TVGB v2 authenticated handshake ve binary frame codec'ini uygular
// # 📌 Modul - Kotlin
// # Version: 0.18.0
// # Aciklama: H2G/G2H key derivation, HMAC-SHA256, 96-byte header ve sliding replay protection saglar
// # Bagimli Oldugu Katman: Tool | Config

package com.turkuazvm.inputagent.tools

import java.io.DataInputStream
import java.io.DataOutputStream
import java.net.Socket
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.security.MessageDigest
import java.security.SecureRandom
import javax.crypto.Mac
import javax.crypto.spec.SecretKeySpec

private const val PROTOCOL_VERSION = 2
private const val HANDSHAKE_RECORD_LEN = 72
private const val HANDSHAKE_PREFIX_LEN = 40
private const val FRAME_HEADER_LEN = 96
private const val FRAME_TAG_LEN = 32
private const val MAX_PAYLOAD_LEN = 1024 * 1024
private const val DIRECTION_H2G = 1
private const val DIRECTION_G2H = 2
private const val MESSAGE_REQUEST = 1
private const val MESSAGE_RESPONSE = 2
private const val FLAGS_NONE = 0
private val CLIENT_MAGIC = byteArrayOf('T'.code.toByte(), 'V'.code.toByte(), 'H'.code.toByte(), '2'.code.toByte())
private val SERVER_MAGIC = byteArrayOf('T'.code.toByte(), 'V'.code.toByte(), 'S'.code.toByte(), '2'.code.toByte())
private val FRAME_MAGIC = byteArrayOf('T'.code.toByte(), 'V'.code.toByte(), 'G'.code.toByte(), 'B'.code.toByte())
private val LABEL_CLIENT_HELLO = "TVGBv2:client-hello".toByteArray(Charsets.UTF_8)
private val LABEL_H2G = "TVGBv2:H2G".toByteArray(Charsets.UTF_8)
private val LABEL_G2H = "TVGBv2:G2H".toByteArray(Charsets.UTF_8)

internal data class TvgbRequestFrame(
    val requestId: Long,
    val payload: ByteArray,
)

internal class TvgbSecureChannel private constructor(
    private val input: DataInputStream,
    private val output: DataOutputStream,
    private val sessionId: ByteArray,
    private val transcriptHash: ByteArray,
    private val hostToGuestKey: ByteArray,
    private val guestToHostKey: ByteArray,
) : AutoCloseable {
    private val replay = ReplayWindow()
    private var nextSequence = 1L

    fun readRequest(): TvgbRequestFrame {
        val header = ByteArray(FRAME_HEADER_LEN)
        input.readFully(header)
        require(header.copyOfRange(0, 4).contentEquals(FRAME_MAGIC)) { "tvgb_frame_magic" }
        val buffer = ByteBuffer.wrap(header).order(ByteOrder.BIG_ENDIAN)
        buffer.position(4)
        require(buffer.short.toInt() and 0xffff == PROTOCOL_VERSION) { "tvgb_frame_version" }
        require(buffer.short.toInt() and 0xffff == FRAME_HEADER_LEN) { "tvgb_frame_header_len" }
        require(buffer.get().toInt() and 0xff == DIRECTION_H2G) { "tvgb_direction" }
        require(buffer.get().toInt() and 0xff == MESSAGE_REQUEST) { "tvgb_message_type" }
        require(buffer.short.toInt() and 0xffff == FLAGS_NONE) { "tvgb_flags" }
        val frameSessionId = ByteArray(16).also { buffer.get(it) }
        require(frameSessionId.contentEquals(sessionId)) { "tvgb_session_id" }
        val sequence = buffer.long
        val requestId = buffer.long
        val payloadLength = buffer.int
        require(payloadLength in 0..MAX_PAYLOAD_LEN) { "tvgb_payload_length" }
        val frameTranscript = ByteArray(32).also { buffer.get(it) }
        require(frameTranscript.contentEquals(transcriptHash)) { "tvgb_transcript" }
        val payload = ByteArray(payloadLength).also { input.readFully(it) }
        val tag = ByteArray(FRAME_TAG_LEN).also { input.readFully(it) }
        verifyHmac(hostToGuestKey, concat(header, payload), tag)
        require(replay.accept(sequence)) { "tvgb_replay" }
        return TvgbRequestFrame(requestId, payload)
    }

    fun writeResponse(requestId: Long, payload: ByteArray) {
        require(payload.size <= MAX_PAYLOAD_LEN) { "tvgb_payload_too_large" }
        val sequence = nextSequence
        nextSequence = if (nextSequence == Long.MAX_VALUE) 1L else nextSequence + 1L
        val header = ByteBuffer.allocate(FRAME_HEADER_LEN).order(ByteOrder.BIG_ENDIAN)
            .put(FRAME_MAGIC)
            .putShort(PROTOCOL_VERSION.toShort())
            .putShort(FRAME_HEADER_LEN.toShort())
            .put(DIRECTION_G2H.toByte())
            .put(MESSAGE_RESPONSE.toByte())
            .putShort(FLAGS_NONE.toShort())
            .put(sessionId)
            .putLong(sequence)
            .putLong(requestId)
            .putInt(payload.size)
            .put(transcriptHash)
            .put(ByteArray(16))
            .array()
        val tag = hmac(guestToHostKey, concat(header, payload))
        output.write(header)
        output.write(payload)
        output.write(tag)
        output.flush()
    }

    override fun close() {
        hostToGuestKey.fill(0)
        guestToHostKey.fill(0)
        sessionId.fill(0)
        transcriptHash.fill(0)
    }

    companion object {
        fun accept(socket: Socket, rootKey: ByteArray): TvgbSecureChannel {
            require(rootKey.size == FRAME_TAG_LEN) { "tvgb_root_key_length" }
            val input = DataInputStream(socket.getInputStream())
            val output = DataOutputStream(socket.getOutputStream())
            val client = ByteArray(HANDSHAKE_RECORD_LEN).also { input.readFully(it) }
            require(client.copyOfRange(0, 4).contentEquals(CLIENT_MAGIC)) { "tvgb_client_magic" }
            val clientVersion = ByteBuffer.wrap(client, 4, 2).order(ByteOrder.BIG_ENDIAN).short.toInt() and 0xffff
            require(clientVersion == PROTOCOL_VERSION) { "tvgb_client_version" }
            verifyHmac(
                rootKey,
                concat(LABEL_CLIENT_HELLO, client.copyOfRange(0, HANDSHAKE_PREFIX_LEN)),
                client.copyOfRange(HANDSHAKE_PREFIX_LEN, HANDSHAKE_RECORD_LEN),
            )

            val serverPrefix = ByteBuffer.allocate(HANDSHAKE_PREFIX_LEN).order(ByteOrder.BIG_ENDIAN)
                .put(SERVER_MAGIC)
                .putShort(PROTOCOL_VERSION.toShort())
                .putShort(0)
                .put(ByteArray(32).also { SecureRandom().nextBytes(it) })
                .array()
            val serverTag = hmac(rootKey, concat(client, serverPrefix))
            val server = concat(serverPrefix, serverTag)
            output.write(server)
            output.flush()

            val transcript = MessageDigest.getInstance("SHA-256").digest(concat(client, server))
            val sessionId = transcript.copyOfRange(0, 16)
            val h2g = hmac(rootKey, concat(LABEL_H2G, transcript))
            val g2h = hmac(rootKey, concat(LABEL_G2H, transcript))
            return TvgbSecureChannel(input, output, sessionId, transcript, h2g, g2h)
        }
    }
}

private class ReplayWindow {
    private var initialized = false
    private var highest = 0L
    private var bitmap = 0L

    fun accept(sequence: Long): Boolean {
        if (sequence <= 0L) return false
        if (!initialized) {
            initialized = true
            highest = sequence
            bitmap = 1L
            return true
        }
        if (sequence > highest) {
            val shift = sequence - highest
            bitmap = if (shift >= 64L) 1L else (bitmap shl shift.toInt()) or 1L
            highest = sequence
            return true
        }
        val delta = highest - sequence
        if (delta >= 64L) return false
        val mask = 1L shl delta.toInt()
        if ((bitmap and mask) != 0L) return false
        bitmap = bitmap or mask
        return true
    }
}

private fun hmac(key: ByteArray, data: ByteArray): ByteArray {
    val mac = Mac.getInstance("HmacSHA256")
    mac.init(SecretKeySpec(key, "HmacSHA256"))
    return mac.doFinal(data)
}

private fun verifyHmac(key: ByteArray, data: ByteArray, expected: ByteArray) {
    require(MessageDigest.isEqual(hmac(key, data), expected)) { "tvgb_authentication" }
}

private fun concat(vararg parts: ByteArray): ByteArray {
    var total = 0
    for (part in parts) total += part.size
    val result = ByteArray(total)
    var offset = 0
    for (part in parts) {
        part.copyInto(result, offset)
        offset += part.size
    }
    return result
}
